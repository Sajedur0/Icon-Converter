//! High-precision, multi-threaded image loading, linear-light resampling and ICO encoding.
//!
//! Accuracy guarantees:
//! - Magic-byte format sniffing + automatic EXIF orientation correction.
//! - Resampling in **premultiplied linear sRGB** (IEC 61966-2-1) with a separable
//!   Lanczos3 filter and analytical subpixel edge coverage — eliminating dark alpha
//!   halos on transparent edges and gamma darkening on fine details.
//! - Bit-exact 1:1 preservation whenever a framed source region already matches the
//!   target icon resolution.
//! - Spec-compliant Windows `.ico` container: 32-bpp BGRA DIB + 1-bpp AND mask for
//!   resolutions `< 256 px`, and 32-bpp RGBA8 PNG for `256 px` and `512 px`.

use anyhow::{bail, Context, Result};
use image::codecs::png::{CompressionType, FilterType as PngFilterType, PngEncoder};
use image::{
    DynamicImage, ExtendedColorType, ImageDecoder, ImageEncoder, ImageFormat, ImageReader,
    RgbaImage,
};
use std::io::Cursor;
use std::path::Path;
use std::sync::OnceLock;

pub const AVAILABLE_SIZES: [u32; 8] = [16, 24, 32, 48, 64, 128, 256, 512];
pub const RULER_SIZES: [u32; 3] = [16, 24, 32];

const LINEAR_LUT_SIZE: usize = 8_192;
const BMP_HEADER_LEN: usize = 40;
const ICO_HEADER_LEN: usize = 6;
const ICO_ENTRY_LEN: usize = 16;

/// How a source image is framed into a square icon canvas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FitMode {
    /// Centre-crop to a square, then resample. Best for photos and full-bleed art.
    Cover,
    /// Fit the whole image inside the square and pad empty space with transparency.
    Contain,
    /// Trim outer fully-transparent borders first, then fit into the square.
    Trim,
}

impl FitMode {
    pub const ALL: [FitMode; 3] = [FitMode::Cover, FitMode::Contain, FitMode::Trim];

    pub const fn index(self) -> usize {
        match self {
            FitMode::Cover => 0,
            FitMode::Contain => 1,
            FitMode::Trim => 2,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            FitMode::Cover => "Fill & crop",
            FitMode::Contain => "Fit & pad",
            FitMode::Trim => "Trim & fit",
        }
    }

    pub const fn hint(self) -> &'static str {
        match self {
            FitMode::Cover => "Non-square images are centre-cropped",
            FitMode::Contain => "Empty space stays transparent",
            FitMode::Trim => "Transparent outer margins are removed",
        }
    }
}

/// Decoded source metadata + prepared pixel buffer ready for instant multi-mode rendering.
pub struct LoadedSource {
    pub prepared: PreparedSource,
    pub format: Option<ImageFormat>,
    pub size_bytes: Option<u64>,
}

/// Source image stored as contiguous RGBA8 along with its non-transparent bounding box.
#[derive(Clone)]
pub struct PreparedSource {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    alpha_bbox: Option<(u32, u32, u32, u32)>,
}

/// One pre-rendered and pre-encoded icon resolution.
#[derive(Clone)]
pub struct PreparedFrame {
    pub size: u32,
    pub image: RgbaImage,
    pub ico_payload: Vec<u8>,
}

/// Complete cached set of all `AVAILABLE_SIZES` for one `FitMode`.
#[derive(Clone)]
pub struct PreparedModeSet {
    pub frames: Vec<PreparedFrame>,
}

impl PreparedModeSet {
    pub fn frame(&self, size: u32) -> Option<&PreparedFrame> {
        self.frames.iter().find(|f| f.size == size)
    }

    /// Exact byte size of the `.ico` file that would be produced for `selected_sizes`.
    pub fn estimated_ico_bytes(&self, selected_sizes: &[u32]) -> u64 {
        if selected_sizes.is_empty() {
            return 0;
        }
        let mut payload_sum = 0u64;
        let mut count = 0u64;
        for &size in selected_sizes {
            if let Some(frame) = self.frame(size) {
                payload_sum += frame.ico_payload.len() as u64;
                count += 1;
            }
        }
        if count == 0 {
            0
        } else {
            ICO_HEADER_LEN as u64 + ICO_ENTRY_LEN as u64 * count + payload_sum
        }
    }

    /// Assembles and writes the pre-encoded `.ico` in a single contiguous disk write.
    pub fn write_ico(&self, selected_sizes: &[u32], output_path: &Path) -> Result<u64> {
        let mut unique: Vec<u32> = selected_sizes.to_vec();
        unique.sort_unstable();
        unique.dedup();
        if unique.is_empty() {
            bail!("Select at least one icon resolution");
        }

        let mut entries: Vec<(u32, &[u8])> = Vec::with_capacity(unique.len());
        for size in unique {
            let frame = self
                .frame(size)
                .with_context(|| format!("Missing prepared icon frame for {size}x{size}"))?;
            entries.push((size, frame.ico_payload.as_slice()));
        }

        let bytes = assemble_ico_bytes(&entries)?;
        let len = bytes.len() as u64;
        std::fs::write(output_path, &bytes).context("Failed to write the ICO file")?;
        Ok(len)
    }
}

impl PreparedSource {
    pub fn from_dynamic(image: DynamicImage) -> Self {
        Self::from_rgba_image(image.into_rgba8())
    }

    fn from_rgba_image(rgba_img: RgbaImage) -> Self {
        // Warm conversion lookup tables once.
        let _ = srgb_to_linear_table();
        let _ = linear_to_srgb_table();

        let (width, height) = (rgba_img.width(), rgba_img.height());
        let rgba = rgba_img.into_raw();
        let alpha_bbox = detect_alpha_bbox(width, height, &rgba);

        Self {
            width,
            height,
            rgba,
            alpha_bbox,
        }
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn alpha_bbox(&self) -> Option<(u32, u32, u32, u32)> {
        self.alpha_bbox
    }

    /// Effective longest side of the active region under `mode`.
    pub fn effective_max_side(&self, mode: FitMode) -> u32 {
        match mode {
            FitMode::Cover => self.width.min(self.height),
            FitMode::Contain => self.width.max(self.height),
            FitMode::Trim => self
                .alpha_bbox
                .map(|(_, _, bw, bh)| bw.max(bh))
                .unwrap_or(0),
        }
    }

    /// Renders a single square RGBA8 frame for `size` and `mode`.
    #[cfg(test)]
    pub fn render_frame(&self, size: u32, mode: FitMode) -> RgbaImage {
        if self.width == 0 || self.height == 0 || size == 0 {
            return RgbaImage::from_pixel(size.max(1), size.max(1), image::Rgba([0, 0, 0, 0]));
        }
        if let Some(exact) = self.try_exact_copy(size, mode) {
            return exact;
        }
        let linear = self.resample_to_linear(size, mode, true);
        linear.to_rgba_image()
    }

    /// Renders and encodes all `AVAILABLE_SIZES` in parallel for instant preview & export.
    pub fn prepare_mode(&self, mode: FitMode) -> PreparedModeSet {
        self.prepare_sizes(&AVAILABLE_SIZES, mode)
    }

    /// Renders and ICO-encodes an arbitrary sorted slice of sizes in parallel.
    pub fn prepare_sizes(&self, sizes: &[u32], mode: FitMode) -> PreparedModeSet {
        let mut sorted: Vec<u32> = sizes.to_vec();
        sorted.sort_unstable();
        sorted.dedup();

        let max_target = sorted.last().copied().unwrap_or(0);
        let eff_span = self.effective_max_side(mode);

        // When the source region is larger than 512 px and we are generating standard
        // multi-resolution icons up to 512 px, use a 32-bit float linear-light pyramid
        // so multi-megapixel images are scanned only once while preserving Nyquist precision.
        if eff_span > 512 && max_target <= 512 && sorted.len() > 1 {
            let master_512 = if eff_span > 1024 {
                self.resample_to_linear(1024, mode, true).downscale(512)
            } else {
                self.resample_to_linear(512, mode, true)
            };

            let need_below_256 = sorted.iter().any(|&s| s < 256);
            let master_256 = if need_below_256 || sorted.contains(&256) {
                Some(master_512.downscale(256))
            } else {
                None
            };

            let frames: Vec<PreparedFrame> = std::thread::scope(|scope| {
                let m512 = &master_512;
                let m256 = master_256.as_ref();
                let handles: Vec<_> = sorted
                    .iter()
                    .map(|&sz| {
                        scope.spawn(move || {
                            let image = if sz == 512 {
                                m512.to_rgba_image()
                            } else if sz == 256 {
                                m256.expect("256 master present").to_rgba_image()
                            } else if sz < 256 {
                                m256.expect("256 master present")
                                    .downscale(sz)
                                    .to_rgba_image()
                            } else {
                                m512.downscale(sz).to_rgba_image()
                            };
                            let ico_payload =
                                encode_ico_entry(&image).expect("valid RGBA8 icon dimensions");
                            PreparedFrame {
                                size: sz,
                                image,
                                ico_payload,
                            }
                        })
                    })
                    .collect();
                handles
                    .into_iter()
                    .map(|h| h.join().expect("icon worker panicked"))
                    .collect()
            });

            return PreparedModeSet { frames };
        }

        // Direct single-pass parallel resampling from the exact source pixels
        // (preserves 1:1 bit-exactness when source matches a target size and avoids
        // double-filtering when upscaling smaller sources).
        let frames: Vec<PreparedFrame> = std::thread::scope(|scope| {
            let handles: Vec<_> = sorted
                .iter()
                .map(|&sz| {
                    scope.spawn(move || {
                        let image = if let Some(exact) = self.try_exact_copy(sz, mode) {
                            exact
                        } else {
                            self.resample_to_linear(sz, mode, false).to_rgba_image()
                        };
                        let ico_payload =
                            encode_ico_entry(&image).expect("valid RGBA8 icon dimensions");
                        PreparedFrame {
                            size: sz,
                            image,
                            ico_payload,
                        }
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().expect("icon worker panicked"))
                .collect()
        });

        PreparedModeSet { frames }
    }

    /// Fast 1:1 bit-exact copy when the framed region is integer-aligned and already `size x size`.
    fn try_exact_copy(&self, size: u32, mode: FitMode) -> Option<RgbaImage> {
        let (rx, ry) = match mode {
            FitMode::Cover => {
                let side = self.width.min(self.height);
                if side != size {
                    return None;
                }
                let dx = self.width - side;
                let dy = self.height - side;
                if dx % 2 != 0 || dy % 2 != 0 {
                    return None;
                }
                (dx / 2, dy / 2)
            }
            FitMode::Contain => {
                if self.width != size || self.height != size {
                    return None;
                }
                (0, 0)
            }
            FitMode::Trim => {
                let (bx, by, bw, bh) = self.alpha_bbox?;
                if bw != size || bh != size {
                    return None;
                }
                (bx, by)
            }
        };

        if rx == 0 && ry == 0 && self.width == size && self.height == size {
            return RgbaImage::from_raw(size, size, self.rgba.clone());
        }

        let dst_row_bytes = (size as usize) * 4;
        let src_row_bytes = (self.width as usize) * 4;
        let mut out = vec![0u8; dst_row_bytes * (size as usize)];
        for y in 0..(size as usize) {
            let src_off = (ry as usize + y) * src_row_bytes + (rx as usize) * 4;
            let dst_off = y * dst_row_bytes;
            out[dst_off..dst_off + dst_row_bytes]
                .copy_from_slice(&self.rgba[src_off..src_off + dst_row_bytes]);
        }
        RgbaImage::from_raw(size, size, out)
    }

    fn resample_to_linear(&self, dst_size: u32, mode: FitMode, parallel: bool) -> LinearSquare {
        let dst = dst_size as usize;
        let Some((xk, yk)) =
            frame_kernels(self.width, self.height, self.alpha_bbox, dst_size, mode)
        else {
            return LinearSquare {
                size: dst_size,
                pixels: vec![[0.0; 4]; dst * dst],
            };
        };

        let mut min_row = usize::MAX;
        let mut max_row = 0usize;
        let mut min_col = usize::MAX;
        let mut max_col = 0usize;
        for i in 0..dst {
            if yk.lengths[i] > 0 {
                min_row = min_row.min(yk.starts[i]);
                max_row = max_row.max(yk.starts[i] + yk.lengths[i] - 1);
            }
            if xk.lengths[i] > 0 {
                min_col = min_col.min(xk.starts[i]);
                max_col = max_col.max(xk.starts[i] + xk.lengths[i] - 1);
            }
        }
        if min_row > max_row || min_col > max_col {
            return LinearSquare {
                size: dst_size,
                pixels: vec![[0.0; 4]; dst * dst],
            };
        }

        let needed_rows = max_row - min_row + 1;
        let needed_cols = max_col - min_col + 1;
        let src_stride_bytes = (self.width as usize) * 4;
        let s2l = srgb_to_linear_table();
        let rgba = self.rgba.as_slice();

        let mut horiz = vec![[0.0f32; 4]; needed_rows * dst];
        let threads = if parallel { worker_threads() } else { 1 };

        let process_rows = |row_base: usize, out_chunk: &mut [[f32; 4]]| {
            let mut row_lin = vec![[0.0f32; 4]; needed_cols];
            let rows = out_chunk.len() / dst;
            for r in 0..rows {
                let y = row_base + r;
                let row_start = y * src_stride_bytes + min_col * 4;
                let row_end = y * src_stride_bytes + (max_col + 1) * 4;
                let src_bytes = &rgba[row_start..row_end];

                for (px_lin, b) in row_lin.iter_mut().zip(src_bytes.chunks_exact(4)) {
                    let a_byte = b[3];
                    if a_byte == 0 {
                        *px_lin = [0.0; 4];
                    } else if a_byte == 255 {
                        *px_lin = [
                            s2l[b[0] as usize],
                            s2l[b[1] as usize],
                            s2l[b[2] as usize],
                            1.0,
                        ];
                    } else {
                        let a = a_byte as f32 * (1.0 / 255.0);
                        *px_lin = [
                            s2l[b[0] as usize] * a,
                            s2l[b[1] as usize] * a,
                            s2l[b[2] as usize] * a,
                            a,
                        ];
                    }
                }

                let dst_row = &mut out_chunk[r * dst..(r + 1) * dst];
                for (x, slot) in dst_row.iter_mut().enumerate() {
                    let len = xk.lengths[x];
                    let cov = xk.coverages[x];
                    if len == 0 || cov <= 0.0 {
                        *slot = [0.0; 4];
                        continue;
                    }
                    let start = xk.starts[x] - min_col;
                    let off = xk.offsets[x];
                    let ws = &xk.weights[off..off + len];
                    let ps = &row_lin[start..start + len];
                    let mut r0 = 0.0f32;
                    let mut g0 = 0.0f32;
                    let mut b0 = 0.0f32;
                    let mut a0 = 0.0f32;
                    for (p, &w) in ps.iter().zip(ws.iter()) {
                        r0 += p[0] * w;
                        g0 += p[1] * w;
                        b0 += p[2] * w;
                        a0 += p[3] * w;
                    }
                    *slot = [r0 * cov, g0 * cov, b0 * cov, a0 * cov];
                }
            }
        };

        if threads > 1 && needed_rows >= 64 {
            let chunk_rows = needed_rows.div_ceil(threads).max(16);
            std::thread::scope(|scope| {
                for (ci, out_chunk) in horiz.chunks_mut(chunk_rows * dst).enumerate() {
                    let row_base = min_row + ci * chunk_rows;
                    scope.spawn(move || process_rows(row_base, out_chunk));
                }
            });
        } else {
            process_rows(min_row, &mut horiz);
        }

        let mut out = vec![[0.0f32; 4]; dst * dst];
        vertical_convolve(&horiz, min_row, dst, &yk, &mut out);
        LinearSquare {
            size: dst_size,
            pixels: out,
        }
    }
}

/// Fast single-read path loader with magic-byte detection and EXIF orientation fix.
pub fn load_source_from_path(path: &Path) -> Result<LoadedSource> {
    let bytes = std::fs::read(path).context("Cannot read the selected image file")?;
    let size_bytes = Some(bytes.len() as u64);
    let (image, format) = decode_bytes_with_hint(&bytes, Some(path))
        .context("The file is corrupt or uses an unsupported image format")?;
    Ok(LoadedSource {
        prepared: PreparedSource::from_dynamic(image),
        format,
        size_bytes,
    })
}

/// Decodes dropped in-memory image bytes with magic-byte detection and EXIF orientation fix.
pub fn load_source_from_bytes(bytes: &[u8]) -> Result<LoadedSource> {
    let (image, format) =
        decode_bytes_with_hint(bytes, None).context("The dropped file is not a readable image")?;
    Ok(LoadedSource {
        prepared: PreparedSource::from_dynamic(image),
        format,
        size_bytes: Some(bytes.len() as u64),
    })
}

fn decode_bytes_with_hint(
    bytes: &[u8],
    path_hint: Option<&Path>,
) -> Result<(DynamicImage, Option<ImageFormat>)> {
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .context("Failed to inspect image header")?;

    if reader.format().is_none() {
        if let Some(fmt) = path_hint.and_then(|p| ImageFormat::from_path(p).ok()) {
            reader.set_format(fmt);
        }
    }

    let detected_format = reader.format();
    let mut decoder = reader.into_decoder()?;
    let orientation = decoder.orientation().ok();
    let mut image = DynamicImage::from_decoder(decoder)?;
    if let Some(orient) = orientation {
        image.apply_orientation(orient);
    }

    if image.width() == 0 || image.height() == 0 {
        bail!("Image has zero width or height");
    }

    Ok((image, detected_format))
}

/// Renders one square, color-accurate RGBA frame for the given icon size.
#[cfg(test)]
pub fn render_frame(img: &DynamicImage, size: u32, mode: FitMode) -> RgbaImage {
    if img.width() == 0 || img.height() == 0 || size == 0 {
        return RgbaImage::from_pixel(size.max(1), size.max(1), image::Rgba([0, 0, 0, 0]));
    }
    let prepared = PreparedSource::from_rgba_image(img.to_rgba8());
    prepared.render_frame(size, mode)
}

/// Writes a single multi-resolution `.ico` file containing every selected size.
#[cfg(test)]
pub fn export_ico(
    img: &DynamicImage,
    sizes: &[u32],
    mode: FitMode,
    output_path: &Path,
) -> Result<()> {
    let prepared = PreparedSource::from_rgba_image(img.to_rgba8());
    let mode_set = prepared.prepare_sizes(sizes, mode);
    mode_set.write_ico(sizes, output_path)?;
    Ok(())
}

/// A sensible default name for the save dialog, based on the source file.
pub fn suggested_output_name(source: Option<&Path>) -> String {
    source
        .and_then(|p| p.file_stem())
        .map(|stem| format!("{}.ico", stem.to_string_lossy()))
        .unwrap_or_else(|| "icon.ico".to_owned())
}

// ============================================================================
// ICO Binary Encoding (32-bpp BGRA DIB for < 256, 32-bpp RGBA8 PNG for >= 256)
// ============================================================================

/// Encodes one square RGBA8 frame according to Microsoft's Windows ICO specification:
/// - `< 256 px`: uncompressed 32-bpp BGRA `BITMAPINFOHEADER` DIB + 1-bpp AND mask.
/// - `>= 256 px` (`256`, `512`, ...): 32-bpp RGBA8 PNG (`color_type = 6`, `bit_depth = 8`).
pub fn encode_ico_entry(frame: &RgbaImage) -> Result<Vec<u8>> {
    let (w, h) = (frame.width(), frame.height());
    if w == 0 || h == 0 {
        bail!("Icon frame dimensions must be non-zero");
    }

    if w >= 256 || h >= 256 {
        encode_png_rgba8(w, h, frame.as_raw())
    } else {
        Ok(encode_bmp_bgra32(w, h, frame.as_raw()))
    }
}

fn encode_png_rgba8(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity((width as usize) * (height as usize));
    let encoder =
        PngEncoder::new_with_quality(&mut buf, CompressionType::Fast, PngFilterType::Adaptive);
    encoder
        .write_image(rgba, width, height, ExtendedColorType::Rgba8)
        .context("Failed to encode PNG icon frame")?;
    Ok(buf)
}

fn encode_bmp_bgra32(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let xor_row_bytes = w * 4;
    let xor_bytes = xor_row_bytes * h;
    let mask_row_bytes = w.div_ceil(32) * 4;
    let mask_bytes = mask_row_bytes * h;

    let mut out = Vec::with_capacity(BMP_HEADER_LEN + xor_bytes + mask_bytes);

    // BITMAPINFOHEADER (40 bytes, little-endian)
    out.extend_from_slice(&(BMP_HEADER_LEN as u32).to_le_bytes()); // biSize
    out.extend_from_slice(&(width as i32).to_le_bytes()); // biWidth
    out.extend_from_slice(&((height * 2) as i32).to_le_bytes()); // biHeight (XOR + AND)
    out.extend_from_slice(&1u16.to_le_bytes()); // biPlanes = 1
    out.extend_from_slice(&32u16.to_le_bytes()); // biBitCount = 32
    out.extend_from_slice(&0u32.to_le_bytes()); // biCompression = BI_RGB
    out.extend_from_slice(&0u32.to_le_bytes()); // biSizeImage
    out.extend_from_slice(&0i32.to_le_bytes()); // biXPelsPerMeter
    out.extend_from_slice(&0i32.to_le_bytes()); // biYPelsPerMeter
    out.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
    out.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant

    // Bottom-up 32-bpp BGRA XOR bitmap
    for row in (0..h).rev() {
        let src = &rgba[row * xor_row_bytes..(row + 1) * xor_row_bytes];
        for px in src.chunks_exact(4) {
            out.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
        }
    }

    // Bottom-up 1-bpp AND mask (DWORD-padded rows; bit = 1 only when alpha == 0)
    let data_bytes_per_row = w.div_ceil(8);
    let pad_bytes = mask_row_bytes - data_bytes_per_row;
    for row in (0..h).rev() {
        let src = &rgba[row * xor_row_bytes..(row + 1) * xor_row_bytes];
        for byte_idx in 0..data_bytes_per_row {
            let mut mask_byte = 0u8;
            let base_x = byte_idx * 8;
            for bit in 0..8 {
                let x = base_x + bit;
                if x >= w {
                    break;
                }
                if src[x * 4 + 3] == 0 {
                    mask_byte |= 1 << (7 - bit);
                }
            }
            out.push(mask_byte);
        }
        out.resize(out.len() + pad_bytes, 0);
    }

    out
}

fn assemble_ico_bytes(entries: &[(u32, &[u8])]) -> Result<Vec<u8>> {
    if entries.is_empty() {
        bail!("Cannot write an empty ICO file");
    }
    if entries.len() > u16::MAX as usize {
        bail!("Too many icon entries");
    }

    let dir_bytes = ICO_HEADER_LEN + ICO_ENTRY_LEN * entries.len();
    let payload_bytes: usize = entries.iter().map(|(_, data)| data.len()).sum();
    let total_bytes = dir_bytes
        .checked_add(payload_bytes)
        .context("ICO file exceeds addressable size")?;
    u32::try_from(total_bytes).context("ICO file exceeds 4 GB format limit")?;

    let mut out = Vec::with_capacity(total_bytes);

    // ICONDIR header (6 bytes)
    out.extend_from_slice(&0u16.to_le_bytes()); // idReserved = 0
    out.extend_from_slice(&1u16.to_le_bytes()); // idType = 1 (Icon)
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes()); // idCount

    // ICONDIRENTRY array (16 bytes * count)
    let mut offset = dir_bytes as u32;
    for &(size, data) in entries {
        let dim_byte = if size >= 256 { 0u8 } else { size as u8 };
        let data_len = u32::try_from(data.len()).context("Single icon frame exceeds 4 GB")?;

        out.push(dim_byte); // bWidth (0 => >= 256)
        out.push(dim_byte); // bHeight (0 => >= 256)
        out.push(0); // bColorCount (0 for 32-bpp)
        out.push(0); // bReserved = 0
        out.extend_from_slice(&1u16.to_le_bytes()); // wPlanes = 1
        out.extend_from_slice(&32u16.to_le_bytes()); // wBitCount = 32
        out.extend_from_slice(&data_len.to_le_bytes()); // dwBytesInRes
        out.extend_from_slice(&offset.to_le_bytes()); // dwImageOffset

        offset = offset
            .checked_add(data_len)
            .context("ICO data offset overflow")?;
    }

    // Raw frame payloads
    for &(_, data) in entries {
        out.extend_from_slice(data);
    }

    Ok(out)
}

// ============================================================================
// Premultiplied Linear-Light sRGB LUTs & Separable Lanczos3 Resampler
// ============================================================================

fn worker_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(16)
}

fn srgb_to_linear_table() -> &'static [f32; 256] {
    static TABLE: OnceLock<[f32; 256]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut t = [0.0f32; 256];
        for (i, slot) in t.iter_mut().enumerate() {
            let s = i as f64 / 255.0;
            let lin = if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            };
            *slot = lin as f32;
        }
        t
    })
}

fn linear_to_srgb_table() -> &'static [u8; LINEAR_LUT_SIZE] {
    static TABLE: OnceLock<[u8; LINEAR_LUT_SIZE]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut t = [0u8; LINEAR_LUT_SIZE];
        let denom = (LINEAR_LUT_SIZE - 1) as f64;
        for (i, slot) in t.iter_mut().enumerate() {
            let lin = i as f64 / denom;
            let s = if lin <= 0.003_130_8 {
                lin * 12.92
            } else {
                1.055 * lin.powf(1.0 / 2.4) - 0.055
            };
            *slot = (s * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
        }
        t
    })
}

#[inline(always)]
fn linear_to_srgb_u8(lin: f32, lut: &[u8; LINEAR_LUT_SIZE]) -> u8 {
    let idx = (lin.clamp(0.0, 1.0) * (LINEAR_LUT_SIZE - 1) as f32 + 0.5) as usize;
    lut[idx]
}

#[inline]
fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-9 {
        1.0
    } else {
        let px = std::f64::consts::PI * x;
        px.sin() / px
    }
}

#[inline]
fn lanczos3(x: f64) -> f64 {
    if x.abs() < 3.0 {
        sinc(x) * sinc(x / 3.0)
    } else {
        0.0
    }
}

struct KernelSet {
    starts: Vec<usize>,
    offsets: Vec<usize>,
    lengths: Vec<usize>,
    weights: Vec<f32>,
    coverages: Vec<f32>,
}

fn build_kernels(
    src_origin: usize,
    src_len: usize,
    dst_len: usize,
    dst_start: f64,
    dst_end: f64,
) -> KernelSet {
    let dst_span = (dst_end - dst_start).max(1e-9);
    let scale = src_len as f64 / dst_span;
    let filter_scale = scale.max(1.0);
    let support = 3.0 * filter_scale;
    let inv_filter_scale = 1.0 / filter_scale;
    let max_idx = src_len.saturating_sub(1) as isize;

    let mut starts = Vec::with_capacity(dst_len);
    let mut offsets = Vec::with_capacity(dst_len);
    let mut lengths = Vec::with_capacity(dst_len);
    let mut coverages = Vec::with_capacity(dst_len);
    let mut weights = Vec::with_capacity(dst_len * ((support as usize) * 2 + 3));

    for out_i in 0..dst_len {
        let cov = ((out_i as f64 + 1.0).min(dst_end) - (out_i as f64).max(dst_start))
            .clamp(0.0, 1.0) as f32;
        if cov <= 0.0 || src_len == 0 {
            starts.push(src_origin);
            offsets.push(weights.len());
            lengths.push(0);
            coverages.push(0.0);
            continue;
        }

        let center = (out_i as f64 + 0.5 - dst_start) * scale - 0.5;
        let left = (center - support).ceil() as isize;
        let right = (center + support).floor() as isize;

        let clamped_first = left.clamp(0, max_idx) as usize;
        let clamped_last = right.clamp(0, max_idx) as usize;
        let span = clamped_last - clamped_first + 1;
        let base_off = weights.len();
        weights.resize(base_off + span, 0.0);

        let mut sum = 0.0f64;
        for tap in left..=right {
            let w = lanczos3((tap as f64 - center) * inv_filter_scale);
            if w != 0.0 {
                let c = tap.clamp(0, max_idx) as usize;
                weights[base_off + (c - clamped_first)] += w as f32;
                sum += w;
            }
        }

        if sum.abs() > 1e-12 {
            let inv = (1.0 / sum) as f32;
            for w in &mut weights[base_off..base_off + span] {
                *w *= inv;
            }
        } else {
            let nearest = (center.round() as isize).clamp(0, max_idx) as usize;
            weights[base_off + (nearest - clamped_first)] = 1.0;
        }

        starts.push(src_origin + clamped_first);
        offsets.push(base_off);
        lengths.push(span);
        coverages.push(cov);
    }

    KernelSet {
        starts,
        offsets,
        lengths,
        weights,
        coverages,
    }
}

fn frame_kernels(
    width: u32,
    height: u32,
    alpha_bbox: Option<(u32, u32, u32, u32)>,
    dst_size: u32,
    mode: FitMode,
) -> Option<(KernelSet, KernelSet)> {
    let dst = dst_size as usize;
    if width == 0 || height == 0 || dst == 0 {
        return None;
    }

    match mode {
        FitMode::Cover => {
            let side = width.min(height) as f64;
            let x_span = dst as f64 * (width as f64 / side);
            let y_span = dst as f64 * (height as f64 / side);
            let x0 = (dst as f64 - x_span) * 0.5;
            let y0 = (dst as f64 - y_span) * 0.5;
            Some((
                build_kernels(0, width as usize, dst, x0, x0 + x_span),
                build_kernels(0, height as usize, dst, y0, y0 + y_span),
            ))
        }
        FitMode::Contain => {
            let max_dim = width.max(height) as f64;
            let scale = dst as f64 / max_dim;
            let dw = width as f64 * scale;
            let dh = height as f64 * scale;
            let x0 = (dst as f64 - dw) * 0.5;
            let y0 = (dst as f64 - dh) * 0.5;
            Some((
                build_kernels(0, width as usize, dst, x0, x0 + dw),
                build_kernels(0, height as usize, dst, y0, y0 + dh),
            ))
        }
        FitMode::Trim => {
            let (bx, by, bw, bh) = alpha_bbox?;
            let max_dim = bw.max(bh) as f64;
            let scale = dst as f64 / max_dim;
            let dw = bw as f64 * scale;
            let dh = bh as f64 * scale;
            let x0 = (dst as f64 - dw) * 0.5;
            let y0 = (dst as f64 - dh) * 0.5;
            Some((
                build_kernels(bx as usize, bw as usize, dst, x0, x0 + dw),
                build_kernels(by as usize, bh as usize, dst, x0, y0 + dh),
            ))
        }
    }
}

struct LinearSquare {
    size: u32,
    pixels: Vec<[f32; 4]>,
}

impl LinearSquare {
    fn downscale(&self, dst_size: u32) -> LinearSquare {
        let src = self.size as usize;
        let dst = dst_size as usize;
        if dst_size == self.size {
            return LinearSquare {
                size: self.size,
                pixels: self.pixels.clone(),
            };
        }

        let k = build_kernels(0, src, dst, 0.0, dst as f64);
        let mut horiz = vec![[0.0f32; 4]; src * dst];

        for y in 0..src {
            let src_row = &self.pixels[y * src..(y + 1) * src];
            let dst_row = &mut horiz[y * dst..(y + 1) * dst];
            for (x, slot) in dst_row.iter_mut().enumerate() {
                let start = k.starts[x];
                let off = k.offsets[x];
                let len = k.lengths[x];
                let ws = &k.weights[off..off + len];
                let ps = &src_row[start..start + len];
                let mut r0 = 0.0f32;
                let mut g0 = 0.0f32;
                let mut b0 = 0.0f32;
                let mut a0 = 0.0f32;
                for (p, &w) in ps.iter().zip(ws.iter()) {
                    r0 += p[0] * w;
                    g0 += p[1] * w;
                    b0 += p[2] * w;
                    a0 += p[3] * w;
                }
                *slot = [r0, g0, b0, a0];
            }
        }

        let mut out = vec![[0.0f32; 4]; dst * dst];
        vertical_convolve(&horiz, 0, dst, &k, &mut out);
        LinearSquare {
            size: dst_size,
            pixels: out,
        }
    }

    fn to_rgba_image(&self) -> RgbaImage {
        let lut = linear_to_srgb_table();
        let mut out = vec![0u8; self.pixels.len() * 4];
        for (px_out, p) in out.chunks_exact_mut(4).zip(self.pixels.iter()) {
            let a = p[3].clamp(0.0, 1.0);
            let a_u8 = (a * 255.0 + 0.5) as u8;
            if a_u8 == 0 {
                px_out.copy_from_slice(&[0, 0, 0, 0]);
            } else if a_u8 == 255 && a >= 0.999 {
                px_out[0] = linear_to_srgb_u8(p[0], lut);
                px_out[1] = linear_to_srgb_u8(p[1], lut);
                px_out[2] = linear_to_srgb_u8(p[2], lut);
                px_out[3] = 255;
            } else {
                let inv_a = 1.0 / a;
                px_out[0] = linear_to_srgb_u8(p[0] * inv_a, lut);
                px_out[1] = linear_to_srgb_u8(p[1] * inv_a, lut);
                px_out[2] = linear_to_srgb_u8(p[2] * inv_a, lut);
                px_out[3] = a_u8;
            }
        }
        RgbaImage::from_raw(self.size, self.size, out)
            .expect("buffer length matches square RGBA8 dimensions")
    }
}

fn vertical_convolve(
    horiz: &[[f32; 4]],
    min_row: usize,
    dst: usize,
    yk: &KernelSet,
    out: &mut [[f32; 4]],
) {
    for out_y in 0..dst {
        let len = yk.lengths[out_y];
        let cov = yk.coverages[out_y];
        if len == 0 || cov <= 0.0 {
            continue;
        }
        let out_row = &mut out[out_y * dst..(out_y + 1) * dst];
        let base = yk.starts[out_y] - min_row;
        let off = yk.offsets[out_y];
        let ws = &yk.weights[off..off + len];
        for (wi, &w) in ws.iter().enumerate() {
            let h_row = &horiz[(base + wi) * dst..(base + wi + 1) * dst];
            for (dst_px, &src_px) in out_row.iter_mut().zip(h_row.iter()) {
                dst_px[0] += src_px[0] * w;
                dst_px[1] += src_px[1] * w;
                dst_px[2] += src_px[2] * w;
                dst_px[3] += src_px[3] * w;
            }
        }
        for px in out_row.iter_mut() {
            let a = (px[3] * cov).clamp(0.0, 1.0);
            px[0] = (px[0] * cov).clamp(0.0, a);
            px[1] = (px[1] * cov).clamp(0.0, a);
            px[2] = (px[2] * cov).clamp(0.0, a);
            px[3] = a;
        }
    }
}

fn detect_alpha_bbox(width: u32, height: u32, rgba: &[u8]) -> Option<(u32, u32, u32, u32)> {
    let w = width as usize;
    let h = height as usize;
    if w == 0 || h == 0 {
        return None;
    }

    let mut top = h;
    for y in 0..h {
        let row = &rgba[y * w * 4..(y + 1) * w * 4];
        if row.chunks_exact(4).any(|px| px[3] != 0) {
            top = y;
            break;
        }
    }
    if top == h {
        return None;
    }

    let mut bottom = top;
    for y in (top..h).rev() {
        let row = &rgba[y * w * 4..(y + 1) * w * 4];
        if row.chunks_exact(4).any(|px| px[3] != 0) {
            bottom = y;
            break;
        }
    }

    let mut left = w;
    let mut right = 0usize;
    for y in top..=bottom {
        let row = &rgba[y * w * 4..(y + 1) * w * 4];
        for x in 0..left {
            if row[x * 4 + 3] != 0 {
                left = x;
                break;
            }
        }
        for x in (right..w).rev() {
            if row[x * 4 + 3] != 0 {
                right = x;
                break;
            }
        }
        if left == 0 && right + 1 == w {
            break;
        }
    }

    Some((
        left as u32,
        top as u32,
        (right - left + 1) as u32,
        (bottom - top + 1) as u32,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::codecs::ico::IcoDecoder;
    use image::Rgba;

    #[test]
    fn available_sizes_include_512() {
        assert_eq!(AVAILABLE_SIZES, [16, 24, 32, 48, 64, 128, 256, 512]);
    }

    #[test]
    fn srgb_linear_round_trip_is_exact() {
        let s2l = srgb_to_linear_table();
        let l2s = linear_to_srgb_table();
        for byte in 0u8..=255 {
            assert_eq!(linear_to_srgb_u8(s2l[byte as usize], l2s), byte);
        }
    }

    #[test]
    fn white_icon_on_transparent_background_has_no_dark_halo() {
        let mut img = RgbaImage::from_pixel(256, 256, Rgba([0, 0, 0, 0]));
        for y in 0..256u32 {
            for x in 0..256u32 {
                let dx = x as f32 - 127.5;
                let dy = y as f32 - 127.5;
                if dx * dx + dy * dy <= 90.0 * 90.0 {
                    img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
            }
        }
        let dyn_img = DynamicImage::ImageRgba8(img);
        let frame = render_frame(&dyn_img, 32, FitMode::Cover);
        for px in frame.pixels() {
            if px[3] > 0 {
                assert!(
                    px[0] >= 254 && px[1] >= 254 && px[2] >= 254,
                    "unexpected dark alpha fringe pixel: {:?}",
                    px.0
                );
            }
        }
    }

    #[test]
    fn exact_size_preserves_pixels_bit_for_bit() {
        let mut img = RgbaImage::new(512, 512);
        for (i, px) in img.pixels_mut().enumerate() {
            *px = Rgba([
                (i % 251) as u8,
                ((i / 7) % 253) as u8,
                ((i / 13) % 249) as u8,
                if i % 5 == 0 { 0 } else { 255 },
            ]);
        }
        let expected = img.clone();
        let out = render_frame(&DynamicImage::ImageRgba8(img), 512, FitMode::Cover);
        assert_eq!(out.as_raw(), expected.as_raw());
    }

    #[test]
    fn contain_mode_is_subpixel_symmetric_on_non_square_images() {
        // 192 x 100 landscape image into 32 x 32 -> vertical span = 16.6667 px
        // Top and bottom anti-aliased boundary rows (y = 7 and y = 24) must have identical alpha.
        let img = RgbaImage::from_pixel(192, 100, Rgba([200, 100, 50, 255]));
        let out = render_frame(&DynamicImage::ImageRgba8(img), 32, FitMode::Contain);
        assert_eq!(out.get_pixel(16, 6)[3], 0);
        assert_eq!(out.get_pixel(16, 25)[3], 0);
        let top_edge_alpha = out.get_pixel(16, 7)[3];
        let bottom_edge_alpha = out.get_pixel(16, 24)[3];
        assert!(top_edge_alpha > 60 && top_edge_alpha < 110);
        assert_eq!(top_edge_alpha, bottom_edge_alpha);
        assert_eq!(out.get_pixel(0, 16)[3], 255);
        assert_eq!(out.get_pixel(31, 16)[3], 255);
    }

    #[test]
    fn trim_mode_removes_transparent_margins() {
        let mut img = RgbaImage::from_pixel(200, 200, Rgba([0, 0, 0, 0]));
        for y in 80..120 {
            for x in 80..120 {
                img.put_pixel(x, y, Rgba([40, 120, 240, 255]));
            }
        }
        let trimmed = render_frame(&DynamicImage::ImageRgba8(img), 40, FitMode::Trim);
        assert_eq!(trimmed.get_pixel(0, 0).0, [40, 120, 240, 255]);
        assert_eq!(trimmed.get_pixel(39, 39).0, [40, 120, 240, 255]);
    }

    #[test]
    fn exported_ico_with_512_matches_spec_and_decodes_cleanly() {
        let mut opaque = RgbaImage::new(600, 400);
        for (y, row) in opaque.enumerate_rows_mut() {
            for (x, _, px) in row {
                *px = Rgba([(x % 256) as u8, (y % 256) as u8, 140, 255]);
            }
        }
        let dyn_img = DynamicImage::ImageRgba8(opaque);
        let prepared = PreparedSource::from_dynamic(dyn_img);
        let mode_set = prepared.prepare_mode(FitMode::Cover);

        let entries: Vec<(u32, &[u8])> = mode_set
            .frames
            .iter()
            .map(|f| (f.size, f.ico_payload.as_slice()))
            .collect();
        let ico_bytes = assemble_ico_bytes(&entries).unwrap();

        // 1. Verify binary ICONDIR + ICONDIRENTRY headers & payloads directly
        assert_eq!(u16::from_le_bytes([ico_bytes[0], ico_bytes[1]]), 0);
        assert_eq!(u16::from_le_bytes([ico_bytes[2], ico_bytes[3]]), 1);
        let count = u16::from_le_bytes([ico_bytes[4], ico_bytes[5]]) as usize;
        assert_eq!(count, AVAILABLE_SIZES.len());

        for (i, &expected_size) in AVAILABLE_SIZES.iter().enumerate() {
            let base = ICO_HEADER_LEN + i * ICO_ENTRY_LEN;
            let expected_dim = if expected_size >= 256 {
                0
            } else {
                expected_size as u8
            };
            assert_eq!(ico_bytes[base], expected_dim);
            assert_eq!(ico_bytes[base + 1], expected_dim);
            assert_eq!(
                u16::from_le_bytes([ico_bytes[base + 4], ico_bytes[base + 5]]),
                1
            );
            assert_eq!(
                u16::from_le_bytes([ico_bytes[base + 6], ico_bytes[base + 7]]),
                32
            );
            let len =
                u32::from_le_bytes(ico_bytes[base + 8..base + 12].try_into().unwrap()) as usize;
            let off =
                u32::from_le_bytes(ico_bytes[base + 12..base + 16].try_into().unwrap()) as usize;
            let slice = &ico_bytes[off..off + len];

            if expected_size >= 256 {
                // PNG magic + IHDR width, height, 8-bit depth, color_type 6 (RGBA8)
                assert_eq!(&slice[0..8], b"\x89PNG\r\n\x1a\n");
                assert_eq!(&slice[12..16], b"IHDR");
                assert_eq!(
                    u32::from_be_bytes(slice[16..20].try_into().unwrap()),
                    expected_size
                );
                assert_eq!(
                    u32::from_be_bytes(slice[20..24].try_into().unwrap()),
                    expected_size
                );
                assert_eq!(slice[24], 8);
                assert_eq!(slice[25], 6);
            } else {
                // BITMAPINFOHEADER: size=40, width=N, height=2N, planes=1, bpp=32
                assert_eq!(u32::from_le_bytes(slice[0..4].try_into().unwrap()), 40);
                assert_eq!(
                    i32::from_le_bytes(slice[4..8].try_into().unwrap()),
                    expected_size as i32
                );
                assert_eq!(
                    i32::from_le_bytes(slice[8..12].try_into().unwrap()),
                    (expected_size * 2) as i32
                );
                assert_eq!(u16::from_le_bytes(slice[14..16].try_into().unwrap()), 32);
            }
        }

        // 2. Verify via `image::codecs::ico::IcoDecoder` (must pick 512x512 32-bpp RGBA8)
        let decoder = IcoDecoder::new(Cursor::new(&ico_bytes)).unwrap();
        assert_eq!(decoder.dimensions(), (512, 512));
        let best = DynamicImage::from_decoder(decoder).unwrap();
        assert_eq!(best.width(), 512);
        assert_eq!(best.height(), 512);

        // 3. Verify `export_ico` end-to-end file write & readback
        let tmp_ico = std::env::temp_dir().join("ico_converter_test_512.ico");
        let dyn_for_disk =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(64, 64, Rgba([10, 20, 30, 255])));
        export_ico(
            &dyn_for_disk,
            &[16, 32, 256, 512],
            FitMode::Contain,
            &tmp_ico,
        )
        .unwrap();
        let disk_bytes = std::fs::read(&tmp_ico).unwrap();
        let _ = std::fs::remove_file(&tmp_ico);
        assert_eq!(u16::from_le_bytes([disk_bytes[4], disk_bytes[5]]), 4);
        let disk_dec = IcoDecoder::new(Cursor::new(&disk_bytes)).unwrap();
        assert_eq!(disk_dec.dimensions(), (512, 512));
    }
}
