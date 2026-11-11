<div align="center">

<img src="icon.png" width="96" alt="ICO Converter logo" />

# ICO Converter

**A minimal, ultra-lightweight desktop utility for converting images into multi-resolution Windows `.ico` files — built with Rust & egui.**

![Rust](https://img.shields.io/badge/Rust-1.75%2B-dea584?style=flat-square&logo=rust&logoColor=white)
![egui](https://img.shields.io/badge/egui-0.28-2563eb?style=flat-square)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-0078d6?style=flat-square)
![License](https://img.shields.io/badge/License-Apache--2.0-green?style=flat-square)

</div>

---

## ✨ Highlights

- ☀️ **Clean, minimal Light-only UI** — compact fixed-size utility window (`476 × 512`, maximize disabled) with refined micro-typography and flat bordered cards.
- 🪶 **Tiny binary & asset footprint** — `egui`/`eframe` default fonts and accessibility bloat disabled, `image` trimmed to essential codecs only, `ico` dependency removed, full LTO + `opt-level = "s"` + symbol stripping, and subsetted Inter fonts (~46 KB total).
- 🖼️ **Pixel-accurate live preview** — framed square transparency preview plus live 16 / 24 / 32 px actual-pixel mini icons.
- 📏 **Canvas Fit modes** — *Crop* (center-crop to square), *Contain* (pad with transparency), or *Auto-Trim* (auto-trim alpha bounds).
- ⚡ **Fast & accurate engine** — multi-threaded premultiplied linear-light Lanczos3 resampling and spec-compliant `.ico` encoding (`16` to `512 px`).

### Specifications

| Feature | Detail |
|---------|--------|
| Input | PNG · APNG · JPG · JPEG · WEBP · BMP · GIF · TIFF · ICO (magic-byte + EXIF orientation aware) |
| Output | Spec-compliant multi-resolution `.ico` (32-bpp BGRA DIB `< 256 px`, 32-bpp RGBA8 PNG `256 & 512 px`) |
| Sizes | `16`, `24`, `32`, `48`, `64`, `128`, `256`, `512` px (toggleable individually) |
| Resampling | Premultiplied linear-light Lanczos3 (halo-free, gamma-accurate) |
| Shortcuts | `Ctrl+O` browse · `Ctrl+Enter` export · `Esc` close dialog |

---

## 🚀 Build & Run

```bash
cargo run                 # development
cargo build --release     # minimal stripped binary in target/release/
```

---

## 📁 Project Structure

```
├── src/
│   ├── main.rs            # Light-only native window setup + app icon
│   ├── app.rs             # State, background loader, shortcuts & layout
│   ├── theme.rs           # Compile-time light palette, subsetted Inter fonts & visuals
│   ├── icons.rs           # Procedural vector icons
│   ├── converter.rs       # Linear-light Lanczos3 resampler + native ICO encoder
│   ├── utils.rs           # Format detection & formatting helpers
│   └── widgets/
│       ├── header.rs      # Minimal top bar
│       ├── drop_zone.rs   # Compact drop target, preview & mini-size strip
│       ├── sizes.rs       # 16–512 px resolution grid & framing selector
│       ├── footer.rs      # Summary & primary export button
│       ├── status.rs      # Inline dismissible status alert
│       └── about.rs       # Compact About modal
├── assets/fonts/          # Subsetted Inter Regular & SemiBold (~46 KB, SIL OFL 1.1)
├── icon.ico               # Application icon
└── build.rs               # Windows icon resource
```

---

## 👤 Developer

- **Sajedur Rahman Roni** — [@Sajedur0](https://github.com/Sajedur0) · [sajedur0.github.io](https://sajedur0.github.io)

## 📄 License

Apache-2.0 · Bundled Inter font subset under [SIL OFL 1.1](assets/fonts/OFL-LICENSE.txt).
