# l3d-ffi

UniFFI bindings for [l3d_rs](https://crates.io/crates/l3d_rs) - L3D luminaire file format parser.

## Supported Languages

UniFFI generates native bindings for:
- **Kotlin** (Android)
- **Swift** (iOS/macOS)
- **Python**
- **Ruby**

## Building

### Build the Rust library

```bash
cargo build --release -p l3d-ffi
```

### Generate bindings

```bash
# Install uniffi-bindgen
cargo install uniffi_bindgen

# Generate Kotlin bindings
uniffi-bindgen generate \
    --library target/release/libl3d_ffi.dylib \
    --language kotlin \
    --out-dir bindings/kotlin

# Generate Swift bindings
uniffi-bindgen generate \
    --library target/release/libl3d_ffi.dylib \
    --language swift \
    --out-dir bindings/swift
```

### For Android

```bash
# Add Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

# Build for Android
cargo build --release -p l3d-ffi --target aarch64-linux-android
cargo build --release -p l3d-ffi --target armv7-linux-androideabi
cargo build --release -p l3d-ffi --target x86_64-linux-android
```

### For iOS

```bash
# Add iOS targets
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim

# Build for iOS
cargo build --release -p l3d-ffi --target aarch64-apple-ios
cargo build --release -p l3d-ffi --target aarch64-apple-ios-sim
```

## Usage

### Kotlin (Android)

```kotlin
import uniffi.l3d_ffi.*

// Parse from bytes
val l3d = L3dFile(fileBytes)

// Or from file path
val l3d = L3dFile.fromPath("/path/to/file.l3d")

// Get geometry parts
val parts = l3d.getParts()
for (part in parts) {
    println("Part: ${part.path}")
    println("Transform: ${part.transform}")
}

// Get assets (OBJ files, textures)
val assets = l3d.getAssets()

// Convert to JSON
val json = l3d.toJson()

// Get version
val version = version()
```

### Swift (iOS/macOS)

```swift
import l3d_ffi

// Parse from bytes
let l3d = try L3dFile(data: fileData)

// Or from file path
let l3d = try L3dFile.fromPath(path: "/path/to/file.l3d")

// Get geometry parts
let parts = l3d.getParts()
for part in parts {
    print("Part: \(part.path)")
    print("Transform: \(part.transform)")
}

// Get assets
let assets = l3d.getAssets()

// Convert to JSON
let json = try l3d.toJson()

// Get version
let ver = version()
```

## API Reference

### `L3dFile`

Main class for parsing and accessing L3D files.

| Method | Description |
|--------|-------------|
| `L3dFile(data)` | Parse L3D from byte array |
| `L3dFile.fromPath(path)` | Parse L3D from file path |
| `getStructureXml()` | Get raw structure.xml content |
| `toJson()` | Convert luminaire data to JSON |
| `getParts()` | Get list of geometry parts |
| `getAssets()` | Get list of asset files |
| `getPartCount()` | Number of geometry parts |
| `getAssetCount()` | Number of asset files |

### `L3dPart`

Geometry part with transformation.

| Field | Type | Description |
|-------|------|-------------|
| `path` | String | Path to OBJ file |
| `transform` | [Float] | 4x4 matrix (16 floats) |

### `L3dAsset`

Asset file from the archive.

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | File path in archive |
| `content` | Bytes | Raw file content |

## License

MIT OR GPL-3.0-or-later
