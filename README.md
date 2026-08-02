<div align="center">

# 🎨 ICO Converter

**A fast, lightweight desktop app to convert any image into a multi-size `.ico` icon — built with Rust & egui**

![Rust](https://img.shields.io/badge/Rust-1.75%2B-dea584?style=flat-square&logo=rust&logoColor=white)
![egui](https://img.shields.io/badge/egui-0.28-8b5cf6?style=flat-square)
![Platform](https://img.shields.io/badge/Platform-Windows-0078d6?style=flat-square&logo=windows&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)

</div>

---

## ✨ Features

- 🖼️ Convert **PNG · JPG · JPEG · BMP · GIF · WEBP · TIFF** → **`.ico`**
- 🖱️ **Drag & drop** or click to browse
- 📐 Select any combination of standard icon sizes (multi-resolution `.ico`)
- 👁️ Live preview with transparency checkerboard
- 🎨 Sleek dark-mode UI
- ⚡ Small, fast, native binary — no runtime dependencies

## 📐 Supported Sizes

| 16×16 | 24×24 | 32×32 | 48×48 | 64×64 | 128×128 | 256×256 |
|:-----:|:-----:|:-----:|:-----:|:-----:|:-------:|:-------:|
| ✔️     | ✔️     | ✔️     | ✔️     | ✔️     | ✔️       | ✔️       |

You can toggle each size independently. All selected sizes are packed into a single `.ico` file.

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) 1.75 or later (stable toolchain)
- Cargo (ships with Rust)
- Windows (a custom `.ico` is embedded into the executable)

### Build & Run

```bash
# Debug build
cargo run

# Optimized release build
cargo build --release

# The binary will be at: target/release/ico_converter.exe
```

## 🎯 Usage

1. **Add an image** — drag & drop it onto the app window, or click **Browse Image**.
2. **Pick sizes** — click the size chips (16 → 256) to include or exclude them.
3. **Convert** — click **Convert & Save ICO**, choose a destination, done. ✅

### Supported Input Formats

| Format  | Extension    |
|---------|--------------|
| PNG     | `.png`       |
| JPEG    | `.jpg`, `.jpeg` |
| BMP     | `.bmp`       |
| GIF     | `.gif`       |
| WebP    | `.webp`      |
| TIFF    | `.tiff`, `.tif` |

> 💡 Tip: transparent PNGs convert cleanly — alpha channel is preserved.

## 🛠️ Tech Stack

- **[eframe / egui](https://github.com/emilk/egui)** — immediate-mode GUI framework
- **[image](https://github.com/image-rs/image)** — image loading & resizing
- **[ico](https://github.com/image-rs/ico)** — ICO container encoding
- **[rfd](https://github.com/PolyMeilex/rfd)** — native file dialogs
- **[winres](https://github.com/mxre/winres)** — embeds `icon.ico` into the `.exe`

## 📁 Project Structure

```
├── src/
│   ├── main.rs        # App entry point & window setup
│   ├── app.rs         # egui UI: layout, theming, interactions
│   ├── converter.rs   # Image loading, resizing & ICO export logic
│   └── utils.rs       # Helpers: extension check, message formatting
├── build.rs           # Embeds the Windows icon resource
├── icon.ico           # App icon
└── Cargo.toml         # Dependencies & release profile
```

## 🏗️ Build Details

The release profile is tuned for a minimal binary:

```toml
[profile.release]
opt-level = "z"   # optimize for size
lto = true        # link-time optimization
strip = true      # strip debug symbols
panic = "abort"   # smaller code, no unwinding
codegen-units = 1 # better optimization
```

## 👤 Developer

| Field | Details |
|-------|---------|
| **Username** | Sajedur0 |
| **GitHub** | [sajedur0](https://github.com/sajedur0) |
| **Website** | [sajedur0.github.io](https://sajedur0.github.io) |

## 📄 License

[MIT](LICENSE) © 2026

---

<div align="center">

**Made with ❤️ in Rust**

</div>
