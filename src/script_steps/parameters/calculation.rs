use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::utils;

#[derive(Debug, Default)]
pub struct Calculation {
    pub calculation: Option<String>,
}

impl Calculation {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> Result<Calculation, String> {
        let mut depth = 1;
        let mut in_text = false;
        let mut item = Calculation { calculation: None };
        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if e.name().as_ref() == b"Text" {
                        in_text = true
                    }
                }
                Ok(Event::Eof) => break,
                Ok(Event::CData(e)) => {
                    if !in_text {
                        continue;
                    }
                    item.calculation = Some(utils::xml_utils::cdata_to_string(&e));
                }
                Ok(Event::End(e)) => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }

                    if e.name().as_ref() == b"Text" {
                        in_text = false
                    }
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(item)
    }

    pub fn display(&self) -> Option<String> {
        match &self.calculation {
            Some(item) => {
                if item.is_empty() {
                    None
                } else {
                    Some(item.clone())
                }
            }
            None => None,
        }
    }
}
