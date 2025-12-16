use super::geometry::{Geometries, Geometry};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Structure {
    pub geometry: Geometry,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Axis {
    #[serde(rename = "@min")]
    min: f64,
    #[serde(rename = "@max")]
    max: f64,
    #[serde(rename = "@step")]
    step: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Vec3f {
    #[serde(rename = "@x")]
    pub x: f32,
    #[serde(rename = "@y")]
    pub y: f32,
    #[serde(rename = "@z")]
    pub z: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rectangle {
    /// Width of the rectangle
    #[serde(rename = "@sizeX")]
    pub size_x: f64,
    /// Height of the rectangle
    #[serde(rename = "@sizeY")]
    pub size_y: f64,
}

impl Rectangle {
    /// Get rectangle dimensions (width, height) in meters
    pub fn size(&self) -> (f64, f64) {
        (self.size_x, self.size_y)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Circle {
    /// Diameter of the circle
    #[serde(rename = "@diameter")]
    pub diameter: f64,
}

impl Circle {
    /// Get circle diameter in meters
    pub fn diameter(&self) -> f64 {
        self.diameter
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Joints {
    pub joint: Vec<Joint>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Joint {
    #[serde(rename = "@partName")]
    pub part_name: String,
    pub position: Vec3f,
    pub rotation: Vec3f,
    pub x_axis: Option<Axis>,
    pub y_axis: Option<Axis>,
    pub z_axis: Option<Axis>,
    pub default_rotation: Option<Vec3f>,
    pub geometries: Geometries,
}
