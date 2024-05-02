use quick_xml::events::Event;
use quick_xml::Reader;

use crate::utils::attributes::get_attribute;

pub fn sanitize(step: &str) -> Option<String> {
    let mut reader = Reader::from_str(step);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) if e.name().as_ref() == b"Step" => {
                match get_attribute(&e, "name") {
                    None => {}
                    Some(value) => {
                        return Some(value.to_string());
                    }
                }
                break;
            }
            _ => {}
        }
        buf.clear()
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let xml = r#"<Step id="79" name="Fenster fixieren" enable="True">"#;

        let expected_output = Some("Fenster fixieren".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
