use crate::utils::attributes::parse_unescaped_attribute;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

#[derive(Debug, Default)]
pub struct DataSourceReference {
    pub id: Option<String>,
    pub name: Option<String>,
}

impl DataSourceReference {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _e: &BytesStart) -> Option<DataSourceReference> {
        let mut depth = 1;
        let mut item = DataSourceReference {
            id: None,
            name: None,
        };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if e.name().as_ref() == b"DataSourceReference" {
                        item.id = parse_unescaped_attribute(&e, "id");
                        item.name = parse_unescaped_attribute(&e, "name");
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
        if let Some(name) = &self.name {
            if self.id == Some(String::from("0")) {
                Some(name.clone())
            } else {
                Some(format!("\"{}\"", name))
            }
        } else {
            None
        }
    }
}
