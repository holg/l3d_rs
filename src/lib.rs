pub mod l3d;
mod tests;

use l3d::Luminaire;
use yaserde::{ser::{to_string_with_config as to_xml_str}, de::{from_str as from_xml_str}};
use serde_json::{from_str as from_json_str, to_string_pretty as to_json_str};
use std::{fs::File as StdFile, path::{Path, PathBuf}, io::Read};
use zip::ZipArchive;
use anyhow::{Context, Result};
use regex::Regex;

/// Normalize XML by removing or collapsing excess whitespace and trimming newlines
pub fn normalize_whitespace(xml: &str) -> String {
    // Remove all excess spaces around self-closing tags
    let re_self_closing = Regex::new(r"(\s+)/>").unwrap();
    let xml = re_self_closing.replace_all(xml, "/>").to_string();

    // Collapse multiple spaces into a single space between tags
    let re_collapse_spaces = Regex::new(r">\s+<").unwrap();
    let xml = re_collapse_spaces.replace_all(&xml, "><").to_string();

    // Trim leading/trailing spaces and newlines from the entire XML
    let xml = xml.trim();

    xml.to_string()
}

pub trait Logger {
    fn log(&self, message: &str);
}

pub trait AsyncLogger {
    fn log(&self, message: &str) -> impl std::future::Future<Output = ()> + Send;
}
#[derive(Clone, Debug)]
pub struct BufFile {
    pub name: Option<String>,
    pub content: Option<Vec<u8>>,
    pub file_id: Option<String>,
    pub content_type: Option<String>,
    pub path: Option<String>,
    pub size: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct FileBufL3d{
    pub files: Vec<BufFile>,
    pub luminaire: Luminaire
}

impl Luminaire {
    pub fn detach(&mut self) -> Result<()> {
        Ok(())
    }
    pub fn remove_bom(s: &str) -> String {
        if s.starts_with("\u{FEFF}") {
            (&s[3..]).to_string()
        } else {
            s.to_string()
        }
    }
    /// a helper function to saintize the XML String
    /// takes a String and returns a String
    /// GldfPRoduct does not really care about the XSD version, so we remove it
    /// and add our own later
    pub fn sanitize_xml_str(xml_str: &str) -> String {
        let cleaned_str = Self::remove_bom(xml_str);
        // Normalize newlines (\r\n to \n)
        let cleaned_str = cleaned_str.replace("\r\n", "\n");
        let re = Regex::new(r"<Luminaire .*?>").unwrap();
        // well we are lazy for now and simple replace the root element with a generic one
        re.replace_all(&cleaned_str, "<Luminaire>").to_string()
    }

    /// Deserialize a Luminaire struct from an XML string
    pub fn from_xml(xml_str: &str) -> Result<Luminaire> {
        let my_xml_str = Self::sanitize_xml_str(&xml_str);
        let result = from_xml_str(&my_xml_str);
        let loaded = result.map_err(anyhow::Error::msg).context("Failed to parse XML string")?;

        Ok(loaded)
    }

    /// Serialize a Luminaire struct into an XML string
    pub fn to_xml(&self) -> Result<String> {
        let yaserde_cfg = yaserde::ser::Config {
            perform_indent: true,
            write_document_declaration: true,
            indent_string: None,
        };
        let x_serialized = to_xml_str(self, &yaserde_cfg).unwrap();
        Ok(x_serialized)
    }

    /// Deserialize a Luminaire struct from a JSON string
    pub fn from_json(json_data: &str) -> Result<Luminaire> {
        let luminaire: Luminaire = from_json_str(json_data)?;
        Ok(luminaire)
    }

    /// Serialize a Luminaire struct into a JSON string
    pub fn to_json(&self) -> Result<String> {
        let json = to_json_str(self)?;
        Ok(json)
    }
    pub fn get_xml_str_from_l3d(path: PathBuf) -> anyhow::Result<String> {
        let zipfile = StdFile::open(path)?;
        let mut zip = ZipArchive::new(zipfile)?;
        let mut xmlfile = zip.by_name("structure.xml")?;
        let mut xml_str = String::new();
        xmlfile.read_to_string(&mut xml_str)?;
        Ok(xml_str)
    }
    /// Read and deserialize a Luminaire struct from an `.l3d` (ZIP) file
    pub fn load_l3d(path: &str) -> anyhow::Result<Luminaire> {
        let path_buf = Path::new(path).to_path_buf();
        let xml_str = Luminaire::get_xml_str_from_l3d(path_buf).map_err(anyhow::Error::msg).context("Failed to parse XML string")?;
        let mut loaded: Luminaire = Luminaire::from_xml(&xml_str)?;
        loaded.path = path.to_string();
        Ok(loaded)
    }
    /// Compare two XML strings after sanitization, normalization, and removal of optional elements
    pub fn compare_xml(raw_xml: &str, generated_xml: &str) -> Result<(), String> {
        // Remove XML declaration
        let raw_xml_clean = remove_xml_declaration(raw_xml);
        let generated_xml_clean = remove_xml_declaration(generated_xml);

        // Sanitize and normalize both XML strings
        let raw_xml_sanitized = Luminaire::sanitize_xml_str(&raw_xml_clean);
        let generated_xml_sanitized = Luminaire::sanitize_xml_str(&generated_xml_clean);

        // Normalize whitespaces
        let raw_xml_normalized = normalize_whitespace(&raw_xml_sanitized);
        let generated_xml_normalized = normalize_whitespace(&generated_xml_sanitized);
        let raw_xml_normalized = remove_specific_empty_elements(&generated_xml_normalized);
        if raw_xml_normalized == generated_xml_normalized {
            Ok(())
        } else {
            Err(format!(
                "The XML strings do not match!\n\nOriginal:\n{}\n\nGenerated:\n{}",
                raw_xml_normalized, generated_xml_normalized
            ))
        }
    }

}


/// Remove the XML declaration
pub fn remove_xml_declaration(xml: &str) -> String {
    xml.replace(r#"<?xml version="1.0" encoding="utf-8"?>"#, "")
}

/// Remove specific empty optional elements like <Name/>
fn remove_specific_empty_elements(xml: &str) -> String {
    // Remove empty <Name/> element
    let re_name = Regex::new(r"<Name\s*/>").unwrap();  // Matches <Name/>
    let xml = re_name.replace_all(xml, "").to_string();

    xml
}
