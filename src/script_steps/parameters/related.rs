use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::script_steps::parameters::layout_reference::LayoutReferenceContainer;
use crate::script_steps::parameters::window_reference::WindowReference;
use crate::utils::attributes::parse_unescaped_attribute;

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
                        "TableOccurrenceReference" => {
                            let table_occurrence = parse_unescaped_attribute(&e, "name")
                                .unwrap_or_else(|| "🚨🚨🚨 <BROKEN REFERENCE> 🚨🚨🚨".to_string());
                            item.parameters.push(format!("Table: {table_occurrence}"));
                        }
                        "LayoutReferenceContainer" => {
                            item.parameters.push(
                                LayoutReferenceContainer::from_xml(reader, &e)
                                    .display()
                                    .unwrap_or_default(),
                            );
                            depth -= 1;
                        }
                        "WindowReference" => {
                            item.parameters.push("New window".to_string());
                            let window = WindowReference::from_xml_without_layout(reader, &e);
                            item.parameters.extend(
                                Some(window.display()).filter(|display| !display.is_empty()),
                            );
                            depth -= 1;
                        }
                        "Options" => {
                            if parse_unescaped_attribute(&e, "ShowRelated").as_deref()
                                == Some("True")
                            {
                                item.parameters.push("Show related".to_string());
                            }
                            if parse_unescaped_attribute(&e, "matchFoundSet").as_deref()
                                == Some("True")
                            {
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

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use super::Related;

    #[test]
    fn new_window_does_not_repeat_layout() {
        let xml = r#"<Parameter type="Related"><TableOccurrenceReference id="1" name="Contacts"></TableOccurrenceReference><LayoutReferenceContainer value="1"><Label>original layout</Label></LayoutReferenceContainer><WindowReference><Style name="Card" value="1"></Style><Name><Calculation datatype="1" position="0"><Calculation><Text><![CDATA["Detail"]]></Text></Calculation></Calculation></Name><LayoutReferenceContainer value="1"><Label>original layout</Label></LayoutReferenceContainer></WindowReference><Options matchFoundSet="True" ShowRelated="True"></Options></Parameter>"#;
        let mut reader = Reader::from_str(xml);
        let Ok(Event::Start(e)) = reader.read_event() else {
            panic!("Wrong read event");
        };
        assert_eq!(
            Related::from_xml(&mut reader, &e).display(),
            r#"Table: Contacts ; Layout: <original layout> ; New window ; Style: Card ; Name: "Detail" ; Show related ; Match found set"#
        );
    }
}
