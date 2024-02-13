use crate::join_scope_id_and_name;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::io::{BufRead, Read};

use crate::utils::attributes::get_attributes;

pub fn parse_script_directories<R: Read + BufRead>(
    reader: &mut Reader<R>,
    _: &BytesStart,
) -> HashMap<String, Vec<String>> {
    let mut script_path: Vec<String> = Vec::new();
    let mut script_id_path_map: HashMap<String, Vec<String>> = HashMap::new();

    let mut depth = 1;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("Error {}", e);
                break;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                depth += 1;

                if depth == 2 && e.name().as_ref() == b"Script" {
                    let mut id = String::new();
                    let mut name = String::new();
                    let mut is_folder = false;
                    let mut is_marker = false;
                    let mut is_separator = false;

                    for attr in get_attributes(&e).unwrap() {
                        match attr.0.as_str() {
                            "id" => id = attr.1.to_string(),
                            "name" => name = attr.1.to_string(),
                            "isFolder" => match attr.1.as_str() {
                                "True" => {
                                    is_folder = true;
                                }
                                "Marker" => {
                                    is_marker = true;
                                }
                                _ => {}
                            },
                            "isSeparatorItem" => is_separator = true,
                            _ => {}
                        }
                    }

                    if is_folder && !name.is_empty() {
                        script_path.push(join_scope_id_and_name(&id, &name));
                    } else if !is_folder && !is_marker && !is_separator {
                        script_id_path_map.insert(id.to_string(), script_path.clone());
                    } else if is_marker {
                        script_path.pop();
                    }
                }
            }
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }

        buf.clear()
    }

    script_id_path_map
}
