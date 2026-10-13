use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::utils::attributes::get_attribute;
use crate::utils::xml_utils::cdata_to_string;

#[derive(Debug, Default)]
struct CfInfo {
    id: String,
    text: String,
}

/// The sanitized, human-readable text of an exploded custom function
pub fn sanitized_custom_function_text(xml_content: &str) -> Option<String> {
    parse_cf_xml(xml_content).map(|cf_info| cf_info.text)
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
                "CustomFunction" => {
                    saw_custom_function = true;
                    if let Some(id) = get_attribute(&e, "id") {
                        cf_info.id = id;
                    }
                }
                "CustomFunctionReference" if !saw_custom_function => {
                    if let Some(id) = get_attribute(&e, "id") {
                        cf_info.id = id;
                    }
                }
                "Calculation" => in_calculation = true,
                "Text" => in_text = true,
                _ => {}
            },
            Ok(Event::CData(e)) if in_text && in_calculation => {
                cf_info.text = cdata_to_string(&e);
                break;
            }
            Ok(Event::End(e)) => match e.name().as_ref() {
                "Calculation" => in_calculation = false,
                "Text" => in_text = false,
                _ => {}
            },
            _ => {}
        }

        buf.clear();
    }

    (!cf_info.id.is_empty()).then_some(cf_info)
}
