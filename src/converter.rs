use image::{DynamicImage, imageops::FilterType};
use std::path::Path;
use anyhow::{Result, Context};

pub const AVAILABLE_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];

pub fn load_image(path: &Path) -> Result<DynamicImage> {
    image::open(path).context("Cannot load image — file is corrupt or unsupported format")
}

pub fn resize_to_sizes(img: &DynamicImage, sizes: &[u32]) -> Vec<DynamicImage> {
    sizes
        .iter()
        .map(|&size| img.resize_exact(size, size, FilterType::Lanczos3))
        .collect()
}

pub fn export_ico(images: Vec<DynamicImage>, output_path: &Path) -> Result<()> {
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for img in images {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let icon_image = ico::IconImage::from_rgba_data(w, h, rgba.into_raw());
        let entry = ico::IconDirEntry::encode(&icon_image)
            .context("Failed to encode ICO entry")?;
        icon_dir.add_entry(entry);
    }

    let file = std::fs::File::create(output_path)
        .context("Cannot create output file")?;
    icon_dir.write(file).context("Failed to write ICO file")?;
    Ok(())
}
