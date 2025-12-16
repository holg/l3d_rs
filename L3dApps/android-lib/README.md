# L3D Android Library

Native libraries and Kotlin bindings for L3D file parsing.

## Structure

```
android-lib/
├── jniLibs/          # Native .so libraries
│   ├── arm64-v8a/    # ARM64 (most modern devices)
│   ├── armeabi-v7a/  # ARMv7 (older devices)
│   ├── x86_64/       # x86_64 (emulator)
│   └── x86/          # x86 (legacy emulator)
├── kotlin/           # UniFFI-generated Kotlin bindings
└── README.md
```

## Building

Run from the repository root:

```bash
# Build for all ABIs
./scripts/build_android.sh release

# Or development build (faster, debug)
./scripts/build_android.sh dev
```

## Usage in Android Project

1. Copy `jniLibs/` to your `app/src/main/` directory
2. Copy Kotlin files from `kotlin/` to your source
3. Add JNA dependency to `build.gradle.kts`:

```kotlin
dependencies {
    implementation("net.java.dev.jna:jna:5.14.0@aar")
}
```

4. Use in your code:

```kotlin
import uniffi.l3d_ffi.*

val l3d = L3dFile(fileBytes)
val parts = l3d.getParts()
val json = l3d.toJson()
```

## Requirements

- Android NDK (set `ANDROID_NDK_HOME` or install via Android Studio)
- cargo-ndk: `cargo install cargo-ndk`
- Rust Android targets (installed via `./scripts/install_all_targets.sh`)

## License

MIT OR GPL-3.0-or-later
