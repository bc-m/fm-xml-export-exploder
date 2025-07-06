use std::fs;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::utils::write_text_file;
use crate::utils::xml_utils::cdata_to_string;

#[derive(Debug, Default)]
struct CfInfo {
    id: String,
    name: String,
    text: String,
}

/// Process all XML files in the cf directory and create sanitized text versions
/// This function mirrors the folder structure of the XML files
pub fn create_sanitized_custom_functions(cf_xml_out_dir_path: &Path, cf_text_out_dir_path: &Path) {
    // Recursively process all XML files in the cf directory
    process_directory_recursively(
        cf_xml_out_dir_path,
        cf_xml_out_dir_path,
        cf_text_out_dir_path,
    );
}

fn process_directory_recursively(
    current_dir: &Path,
    cf_xml_out_dir_path: &Path,
    cf_text_out_dir_path: &Path,
) {
    if let Ok(entries) = fs::read_dir(current_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("xml") {
                process_cf_xml_file(&path, cf_xml_out_dir_path, cf_text_out_dir_path);
            } else if path.is_dir() {
                // Recursively process subdirectories
                process_directory_recursively(&path, cf_xml_out_dir_path, cf_text_out_dir_path);
            }
        }
    }
}

fn process_cf_xml_file(
    xml_file_path: &Path,
    cf_xml_out_dir_path: &Path,
    cf_text_out_dir_path: &Path,
) {
    // Read the XML file content
    let xml_content = match fs::read_to_string(xml_file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", xml_file_path.display(), e);
            return;
        }
    };

    // Parse the script and create sanitized text
    let cf_info = parse_cf_xml(&xml_content);
    if let Some(cf_info) = cf_info {
        // Determine the relative path from the XML file to maintain folder structure
        let relative_path = xml_file_path
            .strip_prefix(cf_xml_out_dir_path)
            .unwrap_or(xml_file_path);
        let output_file_path = cf_text_out_dir_path.join(relative_path);

        // Ensure the output directory exists
        if let Some(parent) = output_file_path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|err| {
                panic!("Error creating directory {}: {}", parent.display(), err)
            });
        }

        // Change extension to .txt
        let output_file_path = output_file_path.with_extension("txt");
        write_text_file(&output_file_path, &cf_info.text);
    }
}

fn parse_cf_xml(xml_content: &str) -> Option<CfInfo> {
    let mut cf_info = CfInfo::default();

    let mut reader = Reader::from_str(xml_content);
    let mut buf = Vec::new();
    let mut depth = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("Error parsing XML: {e}");
                break;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                depth += 1;

                if depth == 2 && e.name().as_ref() == b"CustomFunctionReference" {
                    for attr in crate::utils::attributes::get_attributes(&e).unwrap() {
                        match attr.0.as_str() {
                            "id" => cf_info.id = attr.1.to_string(),
                            "name" => cf_info.name = attr.1.to_string(),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::CData(e)) => {
                cf_info.text = cdata_to_string(&e);
                break;
            }
            Ok(Event::End(_e)) => {
                depth -= 1;

                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }

        buf.clear()
    }

    if cf_info.id.is_empty() {
        None
    } else {
        Some(cf_info)
    }
}
