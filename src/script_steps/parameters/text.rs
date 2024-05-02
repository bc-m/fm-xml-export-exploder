use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::utils::attributes::get_attribute;

#[derive(Debug, Default)]
pub struct Text {
    pub text: Option<String>,
}

impl Text {
    pub fn from_xml(reader: &mut Reader<&[u8]>, e: &BytesStart) -> Result<Text, String> {
        let mut depth = 1;
        let item = Text {
            text: get_attribute(e, "value").map(|text| unescape(&text).unwrap().to_string()),
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
        self.text.clone()
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    use crate::script_steps::parameters::text::Text;

    #[test]
    fn test() {
        let xml = r#"<Text value="a&#13;b&#13;c"></Text>"#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "a\rb\rc".to_string();
        assert_eq!(
            Text::from_xml(&mut reader, &element)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
