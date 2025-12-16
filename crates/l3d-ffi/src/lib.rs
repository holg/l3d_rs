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

use l3d_rs::{from_buffer, L3d, Luminaire};
use std::sync::Arc;

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

/// 3D vector (x, y, z)
#[derive(Debug, Clone, uniffi::Record)]
pub struct L3dVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// A geometry part with its transformation matrix
#[derive(Debug, Clone, uniffi::Record)]
pub struct L3dPart {
    /// Part name from structure.xml
    pub name: String,
    /// Path to the geometry file (e.g., "geom_1/luminaire.obj")
    pub path: String,
    /// Position offset (x, y, z)
    pub position: L3dVec3,
    /// Rotation in degrees (x, y, z)
    pub rotation: L3dVec3,
    /// 4x4 transformation matrix (16 floats, column-major)
    pub transform: Vec<f32>,
}

/// A light emitting object with position and direction
#[derive(Debug, Clone, uniffi::Record)]
pub struct L3dLightEmitter {
    /// Part name
    pub name: String,
    /// Position of the light (x, y, z)
    pub position: L3dVec3,
    /// Rotation/direction of the light (x, y, z in degrees)
    pub rotation: L3dVec3,
    /// Shape type: "rectangle" or "circle"
    pub shape: String,
    /// Width (for rectangle) or diameter (for circle)
    pub size_x: f64,
    /// Height (for rectangle only)
    pub size_y: f64,
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
        let data = std::fs::read(&path).map_err(|e| L3dError::FileError(e.to_string()))?;
        Self::new(data)
    }

    /// Get the raw structure.xml content
    pub fn get_structure_xml(&self) -> String {
        self.inner.file.structure.clone()
    }

    /// Convert the luminaire data to JSON
    pub fn to_json(&self) -> Result<String, L3dError> {
        let luminaire =
            Luminaire::from_xml(&self.inner.file.structure).map_err(|_| L3dError::ParseError)?;
        luminaire
            .to_json()
            .map_err(|e| L3dError::JsonError(e.to_string()))
    }

    /// Get all geometry parts with their transformations
    pub fn get_parts(&self) -> Vec<L3dPart> {
        // Parse the structure to get detailed position/rotation info
        if let Ok(luminaire) = Luminaire::from_xml(&self.inner.file.structure) {
            let mut parts = Vec::new();
            // Start with identity matrix as root transform
            extract_geometry_parts(
                &luminaire.structure.geometry,
                &mut parts,
                &luminaire.geometry_definitions.geometry_file_definition,
                &l3d_rs::MAT4_IDENTITY,
            );
            return parts;
        }

        // Fallback to basic model parts
        self.inner
            .model
            .parts
            .iter()
            .map(|p| L3dPart {
                name: String::new(),
                path: p.path.clone(),
                position: L3dVec3 {
                    x: p.mat[12],
                    y: p.mat[13],
                    z: p.mat[14],
                },
                rotation: L3dVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                transform: p.mat.to_vec(),
            })
            .collect()
    }

    /// Get all light emitting objects
    pub fn get_light_emitters(&self) -> Vec<L3dLightEmitter> {
        let mut emitters = Vec::new();
        if let Ok(luminaire) = Luminaire::from_xml(&self.inner.file.structure) {
            extract_light_emitters(&luminaire.structure.geometry, &mut emitters);
        }
        emitters
    }

    /// Get all asset files (OBJ, textures, etc.)
    pub fn get_assets(&self) -> Vec<L3dAsset> {
        self.inner
            .file
            .assets
            .iter()
            .map(|a| L3dAsset {
                name: a.name.clone(),
                content: a.content.clone(),
            })
            .collect()
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

// ============================================================================
// Helper functions (not exported via UniFFI)
// ============================================================================

/// Recursively extract geometry parts from the structure with accumulated transforms
fn extract_geometry_parts(
    geometry: &l3d_rs::Geometry,
    parts: &mut Vec<L3dPart>,
    defs: &[l3d_rs::GeometryFileDefinition],
    parent_transform: &[f32; 16],
) {
    // Find the geometry file definition
    let geom_id = &geometry.geometry_reference.geometry_id;
    if let Some(def) = defs.iter().find(|d| &d.id == geom_id) {
        let scale = l3d_rs::get_scale(&def.units);

        // Build this geometry's local transform
        let local_transform = l3d_rs::build_transform(&geometry.position, &geometry.rotation);

        // Accumulate with parent transform
        let accumulated = l3d_rs::mat4_mul(parent_transform, &local_transform);

        // Apply scale
        let scale_mat = l3d_rs::mat4_scale(scale);
        let final_transform = l3d_rs::mat4_mul(&accumulated, &scale_mat);

        // Extract world position from accumulated transform (translation is in columns 12,13,14)
        let world_position = L3dVec3 {
            x: accumulated[12],
            y: accumulated[13],
            z: accumulated[14],
        };

        parts.push(L3dPart {
            name: geometry.part_name.clone(),
            path: format!("{}/{}", def.id, def.filename),
            position: world_position,
            rotation: L3dVec3 {
                x: geometry.rotation.x,
                y: geometry.rotation.y,
                z: geometry.rotation.z,
            },
            transform: final_transform.to_vec(),
        });

        // Process joints with accumulated transform (before scale)
        if let Some(joints) = &geometry.joints {
            for joint in &joints.joint {
                // Build joint transform and accumulate
                let joint_transform = l3d_rs::build_transform(&joint.position, &joint.rotation);
                let joint_accumulated = l3d_rs::mat4_mul(&accumulated, &joint_transform);

                for child_geom in &joint.geometries.geometry {
                    extract_geometry_parts(child_geom, parts, defs, &joint_accumulated);
                }
            }
        }
    } else {
        // No geometry definition found, but still process joints
        let local_transform = l3d_rs::build_transform(&geometry.position, &geometry.rotation);
        let accumulated = l3d_rs::mat4_mul(parent_transform, &local_transform);

        if let Some(joints) = &geometry.joints {
            for joint in &joints.joint {
                let joint_transform = l3d_rs::build_transform(&joint.position, &joint.rotation);
                let joint_accumulated = l3d_rs::mat4_mul(&accumulated, &joint_transform);

                for child_geom in &joint.geometries.geometry {
                    extract_geometry_parts(child_geom, parts, defs, &joint_accumulated);
                }
            }
        }
    }
}

/// Recursively extract light emitting objects from the structure
fn extract_light_emitters(geometry: &l3d_rs::Geometry, emitters: &mut Vec<L3dLightEmitter>) {
    // Extract light emitting objects from this geometry
    if let Some(leo) = &geometry.light_emitting_objects {
        // Access light_emitting_object field through JSON (it's private)
        if let Ok(json) = serde_json::to_value(leo) {
            if let Some(objects) = json.get("LightEmittingObject").and_then(|v| v.as_array()) {
                for obj in objects {
                    let name = obj
                        .get("@partName")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let position = obj
                        .get("Position")
                        .map(|p| L3dVec3 {
                            x: p.get("@x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                            y: p.get("@y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                            z: p.get("@z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                        })
                        .unwrap_or(L3dVec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        });

                    let rotation = obj
                        .get("Rotation")
                        .map(|r| L3dVec3 {
                            x: r.get("@x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                            y: r.get("@y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                            z: r.get("@z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                        })
                        .unwrap_or(L3dVec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        });

                    let (shape, size_x, size_y) = if let Some(rect) = obj.get("Rectangle") {
                        (
                            "rectangle".to_string(),
                            rect.get("@sizeX").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            rect.get("@sizeY").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        )
                    } else if let Some(circle) = obj.get("Circle") {
                        (
                            "circle".to_string(),
                            circle
                                .get("@diameter")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0),
                            0.0,
                        )
                    } else {
                        ("unknown".to_string(), 0.0, 0.0)
                    };

                    emitters.push(L3dLightEmitter {
                        name,
                        position,
                        rotation,
                        shape,
                        size_x,
                        size_y,
                    });
                }
            }
        }
    }

    // Recursively process joints
    if let Some(joints) = &geometry.joints {
        for joint in &joints.joint {
            for child_geom in &joint.geometries.geometry {
                extract_light_emitters(child_geom, emitters);
            }
        }
    }
}

/// Get the library version
#[uniffi::export]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
