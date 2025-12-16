use super::structure::{Circle, Rectangle, Vec3f};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct LightEmittingObjects {
    /// List of light emitting objects
    pub light_emitting_object: Vec<LightEmittingObject>,
}

impl LightEmittingObjects {
    /// Get all light emitting objects
    pub fn objects(&self) -> &[LightEmittingObject] {
        &self.light_emitting_object
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct LightEmittingObject {
    /// Part name identifier
    #[serde(rename = "@partName")]
    pub part_name: String,
    /// Position (x, y, z) relative to parent geometry
    pub position: Vec3f,
    /// Rotation (x, y, z) in degrees
    pub rotation: Vec3f,
    /// Rectangle shape (if this LEO is rectangular)
    pub rectangle: Option<Rectangle>,
    /// Circle shape (if this LEO is circular)
    pub circle: Option<Circle>,
}

impl LightEmittingObject {
    /// Get the part name
    pub fn part_name(&self) -> &str {
        &self.part_name
    }

    /// Get position (x, y, z)
    pub fn position(&self) -> (f32, f32, f32) {
        (self.position.x, self.position.y, self.position.z)
    }

    /// Get rotation (x, y, z in degrees)
    pub fn rotation(&self) -> (f32, f32, f32) {
        (self.rotation.x, self.rotation.y, self.rotation.z)
    }

    /// Get rectangle shape if defined
    pub fn rectangle(&self) -> Option<&Rectangle> {
        self.rectangle.as_ref()
    }

    /// Get circle shape if defined
    pub fn circle(&self) -> Option<&Circle> {
        self.circle.as_ref()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LightEmittingObjectReference {
    #[serde(rename = "@lightEmittingPartName")]
    light_emitting_part_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct LightEmittingSurfaces {
    light_emitting_surface: Vec<LightEmittingSurface>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct LightEmittingSurface {
    #[serde(rename = "@partName")]
    part_name: String,
    light_emitting_object_reference: LightEmittingObjectReference,
    face_assignments: FaceAssignments,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct FaceAssignments {
    face_assignment: Option<Vec<FaceAssignment>>,
    face_range_assignment: Option<Vec<FaceRangeAssignment>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FaceAssignment {
    #[serde(rename = "@faceIndex")]
    face_index: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FaceRangeAssignment {
    #[serde(rename = "@faceIndexBegin")]
    face_index_begin: usize,
    #[serde(rename = "@faceIndexEnd")]
    face_index_end: usize,
}
