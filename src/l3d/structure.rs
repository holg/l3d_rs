use serde::{Serialize, Deserialize};
use yaserde_derive::{YaSerialize, YaDeserialize};

/// "Structure" section
#[derive(Debug, Clone, Default, PartialEq, YaSerialize, YaDeserialize, Serialize, Deserialize)]
#[yaserde(rename = "Structure")]
pub struct Structure {
    #[yaserde(child, rename = "Geometry")]
    pub geometries: Vec<GeometryNodeType>,
}

/// "GeometryNodeType" element
#[derive(Debug, Clone, Default, PartialEq, YaSerialize, YaDeserialize, Serialize, Deserialize)]
#[yaserde(rename = "Geometry")]
pub struct GeometryNodeType {
    #[yaserde(attribute, rename = "partName")]
    pub part_name: String,
    #[yaserde(child, rename = "Position")]
    pub position: VectorType,

    #[yaserde(child, rename = "Rotation")]
    pub rotation: VectorType,

    #[yaserde(child, rename = "GeometryReference")]
    pub geometry_reference: GeometryReference,

    #[yaserde(child, rename = "LightEmittingObjects")]
    pub light_emitting_objects: Option<LightEmittingObjects>, // Optional, as it might not always be present
}

/// VectorType used in GeometryNodeType
#[derive(Debug, Clone, Default, PartialEq, YaSerialize, YaDeserialize, Serialize, Deserialize)]
#[yaserde(rename = "VectorType")]
pub struct VectorType {
    #[yaserde(attribute, rename = "x")]
    pub x: f64,

    #[yaserde(attribute, rename = "y")]
    pub y: f64,

    #[yaserde(attribute, rename = "z")]
    pub z: f64,
}

/// GeometryReference element
#[derive(Debug, Clone, Default, PartialEq, YaSerialize, YaDeserialize, Serialize, Deserialize)]
#[yaserde(rename = "GeometryReference")]
pub struct GeometryReference {
    #[yaserde(attribute, rename = "geometryId")]
    pub geometry_id: String,
}

/// LightEmittingObjects container (optional in XSD)
#[derive(Debug, Clone, Default, PartialEq, YaSerialize, YaDeserialize, Serialize, Deserialize)]
#[yaserde(rename = "LightEmittingObjects")]
pub struct LightEmittingObjects {
    #[yaserde(child, rename = "LightEmittingObject")]
    pub light_emitting_object: Vec<LightEmittingObject>,  // Can have multiple LightEmittingObjects
}

/// LightEmittingObject element
#[derive(Debug, Clone, Default, PartialEq, YaSerialize, YaDeserialize, Serialize, Deserialize)]
#[yaserde(rename = "LightEmittingObject")]
pub struct LightEmittingObject {
    #[yaserde(attribute, rename = "partName")]
    pub part_name: String,  // Attribute "partName" in XSD

    #[yaserde(child, rename = "Position")]
    pub position: VectorType,  // Reuse the existing VectorType for Position

    #[yaserde(child, rename = "Rotation")]
    pub rotation: VectorType,  // Reuse the existing VectorType for Rotation

    #[yaserde(child, rename = "Circle")]
    pub circle: Option<Circle>,  // Optional Circle, as defined in the XSD

    #[yaserde(child, rename = "Rectangle")]
    pub rectangle: Option<Rectangle>,  // Optional Rectangle, as defined in the XSD
}

/// Circle element (used in LightEmittingObject)
#[derive(Debug, Clone, Default, PartialEq, YaSerialize, YaDeserialize, Serialize, Deserialize)]
#[yaserde(rename = "Circle")]
pub struct Circle {
    #[yaserde(attribute, rename = "diameter")]
    pub diameter: f64,
}

/// Rectangle element (used in LightEmittingObject)
#[derive(Debug, Clone, Default, PartialEq, YaSerialize, YaDeserialize, Serialize, Deserialize)]
#[yaserde(rename = "Rectangle")]
pub struct Rectangle {
    #[yaserde(attribute, rename = "sizeX")]
    pub size_x: f64,

    #[yaserde(attribute, rename = "sizeY")]
    pub size_y: f64,
}
