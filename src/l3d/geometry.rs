// src/geometry.rs

use serde::{Serialize, Deserialize};
use yaserde_derive::{YaSerialize, YaDeserialize};

/// "GeometryDefinitions" section
#[derive(Debug, Clone, Default, PartialEq, YaSerialize, YaDeserialize, Serialize, Deserialize)]
#[yaserde(rename = "GeometryDefinitions")]
pub struct GeometryDefinitions {
    #[yaserde(child, rename = "GeometryFileDefinition")]
    pub geometry_file_definition: Vec<GeometryFileDefinition>,
}

/// "GeometryFileDefinition" element
#[derive(Debug, Clone, Default, PartialEq, YaSerialize, YaDeserialize, Serialize, Deserialize)]
#[yaserde(rename = "GeometryFileDefinition")]
pub struct GeometryFileDefinition {
    #[yaserde(attribute, rename = "id")]
    pub id: String,

    #[yaserde(attribute, rename = "filename")]
    pub filename: String,

    #[yaserde(attribute, rename = "units")]
    pub units: String, // Example: "mm", "cm", etc.
}
