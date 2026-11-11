<div align="center">

<img src="icon.png" width="112" alt="ICO Converter logo" />

# ICO Converter

**A professional, lightweight desktop utility for creating multi-resolution Windows icons.**

Built with Rust and egui, with accurate image resampling, instant previews, and a minimal native interface.

[![Version](https://img.shields.io/badge/Version-1.0.0-4f46e5?style=flat-square)](CHANGELOG.md)
[![Rust](https://img.shields.io/badge/Rust-Stable-dea584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![egui](https://img.shields.io/badge/egui-0.32-4f46e5?style=flat-square)](https://github.com/emilk/egui)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-64748b?style=flat-square)](https://github.com/Sajedur0/Icon-Converter)
[![License](https://img.shields.io/badge/License-Apache--2.0-15803d?style=flat-square)](LICENSE)

</div>

## Overview

ICO Converter transforms common image formats into a single Windows `.ico` file containing multiple resolutions. Image preparation runs in the background, previews are generated for each canvas mode, and export uses cached frames for fast, consistent results.

## Features

- **Minimal native interface** — fixed `520 × 600` window, soft shadows, refined typography, and the bundled application icon in the header and About dialog.
- **Multi-resolution ICO export** — include any combination of `16`, `24`, `32`, `48`, `64`, `128`, `256`, and `512` px frames.
- **Accurate image processing** — premultiplied linear-light Lanczos3 resampling helps preserve color and transparent edges.
- **Three canvas modes** — Crop, Contain, and Auto-Trim.
- **Live previews** — transparency-aware source preview and pixel-sized `16`, `24`, and `32` px previews.
- **Flexible input** — file picker, drag and drop, magic-byte format detection, and EXIF orientation correction.
- **Fast export** — prepared frames are cached per canvas mode and written as a standards-compliant ICO container.
- **Small distribution** — subsetted Inter fonts, trimmed codec features, LTO, symbol stripping, and size-optimized release builds.

## Download

Download the latest Windows executable from [GitHub Releases](https://github.com/Sajedur0/Icon-Converter/releases). No installer is required; run `ico_converter.exe` directly.

## Usage

1. Drop an image onto the source card or select **Browse**.
2. Choose **Crop**, **Contain**, or **Auto-Trim**.
3. Select the icon resolutions to include.
4. Select **Export .ICO** and choose a destination.

The footer shows the selected canvas mode, resolution count, and estimated output size. After export, the status bar shows the saved path and final file size.

## Format Support

| Feature | Details |
| --- | --- |
| Input | PNG, APNG, JPG, JPEG, WEBP, BMP, GIF, TIFF, ICO |
| Output | Windows `.ico` containing one or more image frames |
| Frames below 256 px | 32-bit BGRA DIB with a 1-bit transparency mask |
| 256 and 512 px frames | 32-bit RGBA PNG |
| Resampling | Premultiplied linear-light Lanczos3 |
| Orientation | Automatic EXIF orientation correction |

## Canvas Modes

| Mode | Behavior |
| --- | --- |
| Crop | Center-crops the source to a square before resizing. |
| Contain | Fits the entire image inside a square with transparent padding. |
| Auto-Trim | Removes fully transparent outer borders, then fits the remaining image. |

## Keyboard Shortcuts

| Shortcut | Action |
| --- | --- |
| `Ctrl+O` | Open an image |
| `Ctrl+Enter` | Export the ICO file |
| `Esc` | Close the About dialog |

## Build from Source

Install the current [stable Rust toolchain](https://rustup.rs/), then run:

```bash
git clone https://github.com/Sajedur0/Icon-Converter.git
cd Icon-Converter
cargo run --release
```

Create an optimized executable with:

```bash
cargo build --locked --release
```

The executable is written to `target/release/`. On Windows, the application icon is embedded through `build.rs`.

### Development Checks

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets
cargo test
```

Linux builds may require the GTK 3 and XKB development libraries used by the native window and file-dialog dependencies.

## Release

Repository releases are built by the Windows release workflow when a version tag is pushed:

```bash
git tag v1.0.0
git push origin v1.0.0
```

See [CHANGELOG.md](CHANGELOG.md) for release history.

## Project Structure

```text
├── .github/workflows/
│   ├── ci.yml               # Formatting, compile, and Clippy checks
│   └── release.yml          # Tagged Windows executable releases
├── assets/fonts/            # Subsetted Inter fonts and font license
├── src/
│   ├── main.rs              # Native window and executable icon
│   ├── app.rs               # State, loading pipeline, actions, and layout
│   ├── converter.rs         # Image resampling and ICO encoding
│   ├── icons.rs             # Procedural interface icons
│   ├── theme.rs             # Palette, typography, shadows, and widgets
│   ├── utils.rs             # Format detection and display helpers
│   └── widgets/             # Header, source, sizes, status, footer, and About
├── build.rs                 # Windows application resources
├── icon.ico                 # Native executable and window icon
├── icon.png                 # Bundled in-app and repository icon
├── CHANGELOG.md             # Release history
├── Cargo.toml               # Package metadata and dependencies
└── LICENSE                  # Apache License 2.0
```

## Author

**Sajedur Rahman Roni**

[GitHub @Sajedur0](https://github.com/Sajedur0) · [sajedur0.github.io](https://sajedur0.github.io)

## License

Copyright © 2026 Sajedur Rahman Roni.

ICO Converter is licensed under the [Apache License, Version 2.0](LICENSE). You may use, modify, and distribute the software in accordance with that license.

The bundled Inter font subset is distributed separately under the [SIL Open Font License 1.1](assets/fonts/OFL-LICENSE.txt).
