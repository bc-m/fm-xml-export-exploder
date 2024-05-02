use quick_xml::events::Event;
use quick_xml::Reader;

use crate::utils::attributes::get_attribute;

pub fn sanitize(step: &str) -> bool {
    let mut enabled = true;

    let mut reader = Reader::from_str(step);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"Step" {
                    match get_attribute(&e, "enable").unwrap().as_str() {
                        "True" => {
                            enabled = true;
                        }
                        "False" => {
                            enabled = false;
                        }
                        _ => {}
                    };
                    break;
                }
            }
            _ => {}
        }
        buf.clear()
    }

    enabled
}
