use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::utils::attributes::parse_unescaped_attribute;

pub struct Comment;

impl Comment {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> String {
        let mut depth = 1;
        let mut comment = String::new();
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;

                    if e.name().as_ref() == "Comment" {
                        comment = parse_unescaped_attribute(&e, "value").unwrap_or_default();
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

        comment
    }
}
