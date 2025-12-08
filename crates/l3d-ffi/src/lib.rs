//! # l3d-ffi
//!
//! UniFFI bindings for l3d_rs - L3D luminaire file format parser.
//!
//! This crate provides cross-language bindings via UniFFI for:
//! - **Kotlin** (Android)
//! - **Swift** (iOS/macOS)
//! - **Python** (alternative to l3d-python)
//! - **Ruby**
//!
//! ## Usage
//!
//! ### Kotlin (Android)
//! ```kotlin
//! val l3d = L3dFile(fileBytes)
//! val parts = l3d.getParts()
//! val json = l3d.toJson()
//! ```
//!
//! ### Swift (iOS)
//! ```swift
//! let l3d = try L3dFile(data: fileData)
//! let parts = l3d.getParts()
//! let json = try l3d.toJson()
//! ```

use std::sync::Arc;
use l3d_rs::{from_buffer, L3d, Luminaire};

uniffi::setup_scaffolding!();

/// Error types for L3D operations
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum L3dError {
    #[error("Failed to parse L3D data")]
    ParseError,
    #[error("Failed to read file: {0}")]
    FileError(String),
    #[error("JSON serialization error: {0}")]
    JsonError(String),
    #[error("Invalid data")]
    InvalidData,
}

/// A geometry part with its transformation matrix
#[derive(Debug, Clone, uniffi::Record)]
pub struct L3dPart {
    /// Path to the geometry file (e.g., "geom_1/luminaire.obj")
    pub path: String,
    /// 4x4 transformation matrix (16 floats, column-major)
    pub transform: Vec<f32>,
}

/// An asset file from the L3D archive
#[derive(Debug, Clone, uniffi::Record)]
pub struct L3dAsset {
    /// File name/path within the archive
    pub name: String,
    /// Raw file contents
    pub content: Vec<u8>,
}

/// Main L3D file interface
#[derive(uniffi::Object)]
pub struct L3dFile {
    inner: L3d,
}

#[uniffi::export]
impl L3dFile {
    /// Parse L3D data from bytes
    #[uniffi::constructor]
    pub fn new(data: Vec<u8>) -> Result<Arc<Self>, L3dError> {
        let inner = from_buffer(&data);
        if inner.file.structure.is_empty() {
            return Err(L3dError::ParseError);
        }
        Ok(Arc::new(Self { inner }))
    }

    /// Parse L3D data from a file path
    #[uniffi::constructor(name = "from_path")]
    pub fn from_path(path: String) -> Result<Arc<Self>, L3dError> {
        let data = std::fs::read(&path)
            .map_err(|e| L3dError::FileError(e.to_string()))?;
        Self::new(data)
    }

    /// Get the raw structure.xml content
    pub fn get_structure_xml(&self) -> String {
        self.inner.file.structure.clone()
    }

    /// Convert the luminaire data to JSON
    pub fn to_json(&self) -> Result<String, L3dError> {
        let luminaire = Luminaire::from_xml(&self.inner.file.structure)
            .map_err(|_| L3dError::ParseError)?;
        luminaire.to_json()
            .map_err(|e| L3dError::JsonError(e.to_string()))
    }

    /// Get all geometry parts with their transformations
    pub fn get_parts(&self) -> Vec<L3dPart> {
        self.inner.model.parts.iter().map(|p| L3dPart {
            path: p.path.clone(),
            transform: p.mat.to_vec(),
        }).collect()
    }

    /// Get all asset files (OBJ, textures, etc.)
    pub fn get_assets(&self) -> Vec<L3dAsset> {
        self.inner.file.assets.iter().map(|a| L3dAsset {
            name: a.name.clone(),
            content: a.content.clone(),
        }).collect()
    }

    /// Get the number of geometry parts
    pub fn get_part_count(&self) -> u64 {
        self.inner.model.parts.len() as u64
    }

    /// Get the number of asset files
    pub fn get_asset_count(&self) -> u64 {
        self.inner.file.assets.len() as u64
    }
}

/// Get the library version
#[uniffi::export]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
