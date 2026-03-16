use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::script_steps::parameters::layout_reference::LayoutReferenceContainer;
use crate::utils::attributes::get_attribute;

#[derive(Debug, Default)]
pub struct Related {
    pub parameters: Vec<String>,
}

impl Related {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> Related {
        let mut depth = 1;
        let mut item = Related::default();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    match e.name().as_ref() {
                        b"TableOccurrenceReference" => {
                            let table_occurrence = get_attribute(&e, "name")
                                .unwrap_or_else(|| "🚨🚨🚨 <BROKEN REFERENCE> 🚨🚨🚨".to_string());
                            item.parameters.push(format!("Table: {table_occurrence}"));
                        }
                        b"LayoutReferenceContainer" => {
                            item.parameters.push(
                                LayoutReferenceContainer::from_xml(reader, &e)
                                    .display()
                                    .unwrap_or_default(),
                            );
                            depth -= 1;
                        }
                        b"WindowReference" => {
                            item.parameters.push("New window".to_string());
                            depth -= 1;
                        }
                        b"Options" => {
                            if get_attribute(&e, "ShowRelated").as_deref() == Some("True") {
                                item.parameters.push("Show related".to_string());
                            }
                            if get_attribute(&e, "matchFoundSet").as_deref() == Some("True") {
                                item.parameters.push("Match found set".to_string());
                            }
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
            buf.clear();
        }

        item
    }

    pub fn display(self) -> String {
        self.parameters.join(" ; ")
    }
}
