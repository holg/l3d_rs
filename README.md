# l3d-rs

[![Crates.io](https://img.shields.io/crates/v/l3d_rs.svg)](https://crates.io/crates/l3d_rs)
[![Documentation](https://docs.rs/l3d_rs/badge.svg)](https://docs.rs/l3d_rs)
[![CI](https://github.com/holg/l3d-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/holg/l3d-rs/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/l3d-rs-python.svg)](https://pypi.org/project/l3d-rs-python/)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

A Rust library for parsing L3D (Luminaire 3D) files, the 3D geometry format used in the lighting industry alongside [GLDF](https://gldf.io) (Global Lighting Data Format).

## Workspace Crates

| Crate | Description | Links |
|-------|-------------|-------|
| [l3d_rs](crates/l3d_rs) | Core L3D parser library | [![crates.io](https://img.shields.io/crates/v/l3d_rs.svg)](https://crates.io/crates/l3d_rs) |
| [l3d-egui](crates/l3d-egui) | 3D Viewer (Desktop & WASM) | [![crates.io](https://img.shields.io/crates/v/l3d-egui.svg)](https://crates.io/crates/l3d-egui) |
| [l3d-rs-python](crates/l3d-python) | Python bindings | [![PyPI](https://img.shields.io/pypi/v/l3d-rs-python.svg)](https://pypi.org/project/l3d-rs-python/) |

## What is L3D?

L3D is a ZIP-based file format containing:
- `structure.xml` - XML file describing the luminaire geometry hierarchy
- OBJ files - 3D geometry files for each part of the luminaire
- Optional texture and material files

The format supports hierarchical assemblies with joints, allowing for adjustable luminaire components (e.g., rotatable lamp heads).

## Quick Start

### Rust

```toml
[dependencies]
l3d_rs = "0.2"
```

```rust
use l3d_rs::from_buffer;

let bytes = std::fs::read("luminaire.l3d").unwrap();
let l3d = from_buffer(&bytes);

for part in &l3d.model.parts {
    println!("{} with {} transform", part.path, part.mat.len());
}
```

### Python

```bash
pip install l3d-rs-python
```

```python
import l3d

data = l3d.from_file("luminaire.l3d")
print(f"Parts: {len(data['model']['parts'])}")
```

### 3D Viewer

Download pre-built binaries from [Releases](https://github.com/holg/l3d-rs/releases) or build from source:

```bash
# Native
cargo run -p l3d-egui

# WASM (requires trunk)
cd crates/l3d-egui && trunk serve
```

## Features

- **XML Parsing**: Parse `structure.xml` into strongly-typed Rust structs
- **JSON Serialization**: Convert between L3D XML and JSON formats
- **3D Model Building**: Automatically compute transformation matrices for rendering
- **No 3D Engine Dependency**: Matrix operations are self-contained (`[f32; 16]`)
- **WASM Compatible**: Works in browsers via WebAssembly
- **Multi-viewport Viewer**: Desktop and web viewer with multi-model support

## Building

```bash
# Build all crates
cargo build --workspace --release

# Run tests
cargo test --workspace

# Build WASM viewer
cd crates/l3d-egui && trunk build --release
```

## License

This project is licensed under the GPL-3.0-or-later license.

## Related Projects

- [GLDF](https://gldf.io) - Global Lighting Data Format
- [L3D Specification](https://github.com/globallightingdata/l3d) - Official L3D format specification
- [gldf-rs](https://github.com/globallightingdata/gldf-rs) - GLDF parser for Rust
