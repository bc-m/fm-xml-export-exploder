use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::io::{BufRead, Read};

use crate::utils::attributes::get_attributes;

pub fn parse_base_table_catalog<R: Read + BufRead>(
    reader: &mut Reader<R>,
    _: &BytesStart,
) -> HashMap<String, String> {
    let mut depth = 1;
    let mut table_name_id_map: HashMap<String, String> = HashMap::new();

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

                if depth == 2 && e.name().as_ref() == b"BaseTable" {
                    let mut id = String::new();
                    let mut name = String::new();

                    for attr in get_attributes(&e).unwrap() {
                        match attr.0.as_str() {
                            "id" => id = attr.1.to_string(),
                            "name" => name = attr.1.to_string(),
                            _ => {}
                        }
                    }

                    table_name_id_map.insert(name, id);
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

    table_name_id_map
}
