# L3D Apps

Platform-specific libraries and applications for L3D file parsing.

## Structure

```
L3dApps/
├── SPM-L3dKit/      # Swift Package Manager package for Apple platforms
│   ├── Package.swift
│   ├── Sources/L3dKit/
│   ├── Tests/L3dKitTests/
│   └── L3dFfi.xcframework (generated)
│
└── android-lib/     # Android native libraries and Kotlin bindings
    ├── jniLibs/     # Native .so files for each ABI
    ├── kotlin/      # UniFFI-generated Kotlin bindings
    └── README.md
```

## Building

### Apple (Swift Package Manager)

```bash
# Install required targets
./scripts/install_all_targets.sh

# Build SPM package (dev or release)
./scripts/build_spm.sh release

# Or build universal package with debug + release
./scripts/build_spm_universal.sh
```

### Android

```bash
# Install required targets
./scripts/install_all_targets.sh

# Build Android libraries
./scripts/build_android.sh release
```

## Usage

### Swift/iOS/macOS

```swift
import L3dKit

let l3d = try L3dFile(data: fileData)
let parts = l3d.getParts()
let json = l3d.toJson()
```

### Kotlin/Android

```kotlin
import uniffi.l3d_ffi.*

val l3d = L3dFile(fileBytes)
val parts = l3d.getParts()
val json = l3d.toJson()
```

## Requirements

### Apple Platforms
- Xcode 15+
- Swift 5.9+
- Rust with targets: aarch64-apple-darwin, x86_64-apple-darwin, aarch64-apple-ios, aarch64-apple-ios-sim

### Android
- Android NDK (installed via Android Studio)
- cargo-ndk (`cargo install cargo-ndk`)
- Rust with targets: aarch64-linux-android, armv7-linux-androideabi, x86_64-linux-android, i686-linux-android

## License

MIT OR GPL-3.0-or-later
