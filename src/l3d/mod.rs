// src/lib.rs

pub mod header;
pub mod geometry;
pub mod structure;

use serde::{Deserialize, Serialize};
use yaserde_derive::{YaDeserialize, YaSerialize};

pub use header::LuminaireHeader;
pub use geometry::GeometryDefinitions;
pub use structure::Structure;

/// Root element "Luminaire"
#[derive(Debug, Clone, Default, PartialEq, YaSerialize, YaDeserialize, Serialize, Deserialize)]
#[yaserde(
    rename = "Luminaire",
    namespace = "ns1: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "ns1:noNamespaceSchemaLocation: https://gldf.io/xsd/l3d/0.11.0/l3d.xsd",
    ns1: noNamespaceSchemaLocation = "https://gldf.io/xsd/l3d/0.11.0/l3d.xsd"

)]
pub struct Luminaire {
    /// This field is not serialized or deserialized but can be used to store the path to the
    /// L3d luminaire file.
    #[serde(skip_serializing, skip_deserializing)]
    #[yaserde(skip_serializing, skip_deserializing)]
    pub path: String,
    #[yaserde(child, rename = "Header")]
    pub header: LuminaireHeader,

    #[yaserde(child, rename = "GeometryDefinitions")]
    pub geometry_definitions: GeometryDefinitions,

    #[yaserde(child, rename = "Structure")]
    pub structure: Structure,
}
