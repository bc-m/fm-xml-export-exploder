use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::utils::attributes::parse_unescaped_attribute;

#[derive(Debug, Default)]
pub struct ScriptReference {
    pub data_source_name: Option<String>,
    pub script_name: Option<String>,
}

impl ScriptReference {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> ScriptReference {
        let mut depth = 1;
        let mut item = ScriptReference::default();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    match e.name().as_ref() {
                        b"DataSourceReference" => {
                            item.data_source_name = parse_unescaped_attribute(&e, "name")
                        }
                        b"ScriptReference" => {
                            item.script_name = parse_unescaped_attribute(&e, "name")
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

    pub fn display(self) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(name) = self.script_name {
            parts.push(format!("\"{name}\""));
        }
        if let Some(name) = self.data_source_name {
            parts.push(format!("from file \"{name}\""));
        }
        (!parts.is_empty()).then(|| parts.join(" "))
    }
}
