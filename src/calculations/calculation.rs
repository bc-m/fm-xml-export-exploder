use crate::utils;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

#[derive(Debug, Default)]
pub struct Calculation {}

impl Calculation {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> Result<String, String> {
        let mut depth = 1;
        let mut in_text = false;
        let mut calculation = String::new();
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
                    calculation = utils::xml_utils::cdata_to_string(&e);
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

        Ok(calculation)
    }
}
