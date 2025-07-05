use crate::utils::attributes::parse_unescaped_attribute;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

#[derive(Debug, Default)]
pub struct ScriptReference {
    pub data_source_name: Option<String>,
    pub script_name: Option<String>,
}

impl ScriptReference {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _e: &BytesStart) -> Option<ScriptReference> {
        let mut depth = 1;
        let mut item = ScriptReference {
            data_source_name: None,
            script_name: None,
        };

        let mut buf: Vec<u8> = Vec::new();
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

        Some(item)
    }

    pub fn display(&self) -> Option<String> {
        let mut parameters = vec![];

        if let Some(script_name) = &self.script_name {
            parameters.push(format!("\"{script_name}\""));
        }
        if let Some(data_source_name) = &self.data_source_name {
            parameters.push(format!("from file \"{data_source_name}\""));
        }

        if !parameters.is_empty() {
            Some(parameters.join(" "))
        } else {
            None
        }
    }
}
