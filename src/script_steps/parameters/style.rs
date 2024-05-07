use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::utils::attributes::get_attribute;

#[derive(Debug, Default)]
pub struct Style {
    pub style: Option<String>,
}

impl Style {
    pub fn from_xml(reader: &mut Reader<&[u8]>, e: &BytesStart) -> Result<Style, String> {
        let mut depth = 1;
        let item = Style {
            style: get_attribute(e, "name"),
        };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(_)) => {
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

        Ok(item)
    }

    pub fn display(&self) -> Option<String> {
        Some(format!("Style: {}", self.style.clone().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    use crate::script_steps::parameters::style::Style;

    #[test]
    fn test() {
        let xml = r#"<Style name="Dokument" value="3221291010"></Style>"#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "Style: Dokument".to_string();
        assert_eq!(
            Style::from_xml(&mut reader, &element)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
