// src/header.rs

use serde::{Serialize, Deserialize};
use yaserde_derive::{YaSerialize, YaDeserialize};

/// The "Header" section of the Luminaire
#[derive(Debug, Clone, Default, PartialEq, YaSerialize, YaDeserialize, Serialize, Deserialize)]
#[yaserde(rename = "Header")]
pub struct LuminaireHeader {
    #[yaserde(child, rename = "Name")]
    pub name: Option<String>,

    #[yaserde(child, rename = "Description")]
    pub description: Option<String>,

    #[yaserde(child, rename = "CreatedWithApplication")]
    pub created_with_application: String,

    #[yaserde(child, rename = "CreationTimeCode")]
    pub creation_time_code: String,

    #[yaserde(child, rename = "FormatVersion")]
    pub format_version: FormatVersion,
}

#[derive(Debug, Clone, Default, PartialEq, YaSerialize, YaDeserialize, Serialize, Deserialize)]
#[yaserde(rename = "FormatVersion")]
pub struct FormatVersion {
    #[yaserde(attribute, rename = "major")]
    pub major: i32,

    #[yaserde(attribute, rename = "minor")]
    pub minor: i32,

    #[yaserde(attribute, rename = "pre-release")]
    pub pre_release: Option<i32>,
}
