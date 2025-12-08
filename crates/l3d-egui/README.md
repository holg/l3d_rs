# l3d-egui

[![Crates.io](https://img.shields.io/crates/v/l3d-egui.svg)](https://crates.io/crates/l3d-egui)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

A 3D viewer for L3D (Luminaire 3D) files, built with [three-d](https://github.com/asny/three-d) and egui. Works on desktop (Windows, macOS, Linux) and in the browser via WebAssembly.

## Features

- **Cross-platform**: Native desktop and WASM/WebGL support
- **Multi-viewport**: View multiple L3D files side-by-side in a grid layout
- **Interactive**: Rotate, zoom, and pan with mouse controls
- **File loading**: Open files via menu, drag-and-drop, or file picker

## Installation

### Pre-built binaries

Download from [GitHub Releases](https://github.com/holg/l3d-rs/releases).

### From crates.io

```bash
cargo install l3d-egui
```

### From source

```bash
# Native
cargo run -p l3d-egui --release

# WASM (requires trunk: cargo install trunk)
cd crates/l3d-egui
trunk serve --release
```

## Usage

### Desktop

```bash
# Run viewer
l3d-viewer

# Or with a file
l3d-viewer path/to/luminaire.l3d
```

### Controls

- **Drag**: Rotate camera
- **Scroll**: Zoom
- **Shift+Drag**: Pan
- **Click viewport**: Select active viewer (multi-viewport mode)

### Menu Options

- **File > Open...**: Replace current model
- **File > Add to Scene...**: Add model to current viewport
- **File > Open in New Viewer...**: Create new viewport with model
- **File > Clear All**: Remove all models

## Building WASM

```bash
cd crates/l3d-egui
trunk build --release
```

The output will be in `dist/` - serve with any static file server.

## License

GPL-3.0-or-later

## Related

- [l3d_rs](https://crates.io/crates/l3d_rs) - Core L3D parser library
- [l3d-rs-python](https://pypi.org/project/l3d-rs-python/) - Python bindings
