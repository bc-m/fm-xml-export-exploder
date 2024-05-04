use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::utils::attributes::get_attribute;

#[derive(Debug, Default)]
pub struct Comment {}

impl Comment {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> Result<String, String> {
        let mut depth = 1;
        let mut comment = String::new();
        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Start(e)) => {
                    depth += 1;

                    if e.name().as_ref() == b"Comment" {
                        comment = get_attribute(&e, "value").unwrap_or_default();
                    };
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

        Ok(unescape(&comment).unwrap().to_string())
    }
}
