use quick_xml::Reader;
use quick_xml::events::Event;

use crate::utils::attributes::get_attribute;

pub fn sanitize(step: &str) -> bool {
    let mut reader = Reader::from_str(step);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) if e.name().as_ref() == b"Step" => {
                return get_attribute(&e, "enable").unwrap() != "False";
            }
            _ => {}
        }
        buf.clear()
    }
    true
}
