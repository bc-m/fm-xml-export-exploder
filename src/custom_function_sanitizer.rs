use std::fs;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::utils::attributes::get_attribute;
use crate::utils::file_utils::for_each_xml_file;
use crate::utils::write_text_file;
use crate::utils::xml_utils::cdata_to_string;

#[derive(Debug, Default)]
struct CfInfo {
    id: String,
    text: String,
}

/// Process all XML files in the cf directory and create sanitized text versions
/// This function mirrors the folder structure of the XML files
pub fn create_sanitized_custom_functions(cf_xml_out_dir_path: &Path, cf_text_out_dir_path: &Path) {
    for_each_xml_file(
        cf_xml_out_dir_path,
        cf_xml_out_dir_path,
        cf_text_out_dir_path,
        &mut |xml_file_path, output_file_path| {
            let Ok(xml_content) = fs::read_to_string(xml_file_path) else {
                eprintln!("Error reading file {}", xml_file_path.display());
                return;
            };
            if let Some(cf_info) = parse_cf_xml(&xml_content) {
                write_text_file(output_file_path, &cf_info.text);
            }
        },
    );
}

fn parse_cf_xml(xml_content: &str) -> Option<CfInfo> {
    let mut cf_info = CfInfo::default();

    let mut reader = Reader::from_str(xml_content);
    let mut buf = Vec::new();
    let mut saw_custom_function = false;
    let mut in_calculation = false;
    let mut in_text = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("Error parsing XML: {e}");
                break;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"CustomFunction" => {
                    saw_custom_function = true;
                    if let Some(id) = get_attribute(&e, "id") {
                        cf_info.id = id;
                    }
                }
                b"CustomFunctionReference" if !saw_custom_function => {
                    if let Some(id) = get_attribute(&e, "id") {
                        cf_info.id = id;
                    }
                }
                b"Calculation" => in_calculation = true,
                b"Text" => in_text = true,
                _ => {}
            },
            Ok(Event::CData(e)) => {
                if in_text && in_calculation {
                    cf_info.text = cdata_to_string(&e);
                    break;
                }
            }
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"Calculation" => in_calculation = false,
                b"Text" => in_text = false,
                _ => {}
            },
            _ => {}
        }

        buf.clear();
    }

    (!cf_info.id.is_empty()).then_some(cf_info)
}
