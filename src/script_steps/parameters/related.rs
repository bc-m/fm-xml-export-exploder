use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::script_steps::parameters::layout_reference::LayoutReferenceContainer;
use crate::utils::attributes::get_attribute;

#[derive(Debug, Default)]
pub struct Related {
    pub parameters: Vec<String>,
}

impl Related {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> Option<Related> {
        let mut depth = 1;
        let mut item = Related {
            parameters: Vec::new(),
        };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;

                    let element_name = e.name();
                    match element_name.as_ref() {
                        b"TableOccurrenceReference" => {
                            let table_occurrence = get_attribute(&e, "name")
                                .unwrap_or("🚨🚨🚨 <BROKEN REFERENCE> 🚨🚨🚨".to_string());
                            item.parameters.push(format!("Table: {}", table_occurrence));
                        }
                        b"LayoutReferenceContainer" => {
                            item.parameters.push(
                                LayoutReferenceContainer::from_xml(reader, &e)
                                    .unwrap()
                                    .display()
                                    .unwrap_or("".to_string()),
                            );
                            depth -= 1;
                        }
                        b"WindowReference" => {
                            item.parameters.push("New window".to_string());
                            depth -= 1;
                        }
                        b"Options" => {
                            if let Some(show_related) = get_attribute(&e, "ShowRelated") {
                                if show_related == "True" {
                                    item.parameters.push("Show related".to_string())
                                }
                            };
                            if let Some(match_found_set) = get_attribute(&e, "matchFoundSet") {
                                if match_found_set == "True" {
                                    item.parameters.push("Match found set".to_string())
                                }
                            };
                        }
                        _ => {}
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

        Some(item)
    }

    pub fn display(&self) -> Option<String> {
        Some(self.parameters.join(" ; "))
    }
}
