[![Rust](https://github.com/holg/l3d_rs/actions/workflows/rust.yml/badge.svg)](https://github.com/holg/l3d_rs/actions/workflows/rust.yml)
# `l3d_rs` - A Rust Library for Handling L3D Files

`l3d_rs` is a Rust library designed to read, parse, and manipulate `.l3d` files, which are ZIP containers that include structured XML files along with other assets such as geometries. This library provides functionality to deserialize `.l3d` files into usable Rust structures, serialize them back to XML, and perform JSON conversions for flexibility.
(https://github.com/globallightingdata/l3d)

## Features

- **Read `.l3d` Files**: Extract and parse `.l3d` ZIP containers.
- **Handle XML and Assets**: Work with the XML data and related assets (e.g., geometries, textures) inside `.l3d` files.
- **XML and JSON Support**: Serialize/deserialize `.l3d` content to/from both XML and JSON formats.
- **XML Comparison**: Compare extracted and generated XML to ensure data integrity.

## Installation

Add `l3d_rs` to your `Cargo.toml` dependencies:

```toml
[dependencies]
l3d_rs = "0.1.0"
```

## Usage
# Reading an `.l3d` File
You can read and deserialize an `.l3d` file (which is a ZIP container) into the Luminaire struct.
```rust
use l3d_rs::Luminaire;

fn main() {
    let file_path = "path/to/your/file.l3d";

    // Load and parse the .l3d file
    let luminaire = Luminaire::load_l3d(file_path).expect("Failed to load L3D file");

    // Work with the luminaire data
    println!("{:?}", luminaire);
}
```
## Serializing to XML
### Once you've loaded the `.l3d` file, you can serialize the data back into XML:
```rust
let luminaire_xml = luminaire.to_xml().expect("Failed to serialize to XML");
println!("{}", luminaire_xml);
```

## Serializing to JSON
### You can also serialize the luminaire data into `JSON` for easy integration with web or other systems:

```rust
let luminaire_json = luminaire.to_json().expect("Failed to serialize to JSON");
println!("{}", luminaire_json);
```

## Deserializing from JSON
### You can deserialize JSON data back into the Luminaire structure, allowing for easy round-tripping between JSON and XML formats:

```rust
let json_data = r#"{ "your": "json_data_here" }"#;

let luminaire_from_json: Luminaire = Luminaire::from_json(json_data).expect("Failed to deserialize from JSON");

println!("{:?}", luminaire_from_json);
```
