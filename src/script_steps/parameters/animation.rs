use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::utils::attributes::get_attribute;

#[derive(Debug, Default)]
pub struct Animation {
    pub value: Option<String>,
}

impl Animation {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> Animation {
        let mut depth = 1;
        let mut item = Animation::default();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    if e.name().as_ref() == b"Animation" {
                        item.value = get_attribute(&e, "name");
                    }
                    depth += 1;
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
        self.value.map(|value| format!("Animation: {value}"))
    }
}
