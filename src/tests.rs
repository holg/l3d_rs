use crate::l3d::{Luminaire};
#[test]
fn test_l3d_raw_xml_vs_to_xml() {
    // Path to the .l3d file (which is XML format)
    let file_path = "tests/data/recessed_round.l3d";
    // Load and deserialize the .l3d file into Luminaire
    let loaded = Luminaire::load_l3d(file_path).expect("Failed to load L3D file");
    // Reserialize the Luminaire struct back into XML
    let loaded_to_xml = loaded.to_xml().expect("Failed to serialize to XML");
    let raw_xml = Luminaire::get_xml_str_from_l3d(file_path.into()).expect("Failed to load L3D structure.xml file");
    // Compare the raw XML and generated XML
    if let Err(err) = Luminaire::compare_xml(&raw_xml, &loaded_to_xml) {
        panic!("{}", err);  // This will panic with the two XML strings for easy comparison in IDE
    }
}
#[test]
fn test_l3d_json_vs_to_xml() {
    // Path to the .l3d file (which is XML format)
    let file_path = "tests/data/recessed_round.l3d";
    // Load and deserialize the .l3d file into Luminaire
    let loaded = Luminaire::load_l3d(file_path).expect("Failed to load L3D file");
    // Reserialize the Luminaire struct back into XML
    let loaded_to_xml = loaded.to_xml().expect("Failed to serialize to XML");
    let json_data = loaded.to_json().expect("Failed to serialize to JSON");

    // Step 6: Deserialize the JSON back into the Rust struct
    let luminaire_from_json: Luminaire = Luminaire::from_json(&json_data).expect("Failed to deserialize from JSON");

    // Step 7: Serialize the Luminaire struct (from JSON) back to XML
    let xml_from_json = Luminaire::to_xml(&luminaire_from_json).expect("Failed to serialize to XML");
    if let Err(err) = Luminaire::compare_xml(&xml_from_json, &loaded_to_xml) {
        panic!("{}", err);  // This will panic with the two XML strings for easy comparison in IDE
    }
}
