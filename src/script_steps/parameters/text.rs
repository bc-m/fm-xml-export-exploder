use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::BytesStart;

use crate::utils::attributes::get_attribute;
use crate::utils::xml_utils::skip_rest_of_element;

#[derive(Debug, Default)]
pub struct Text {
    pub text: Option<String>,
}

impl Text {
    pub fn from_xml(reader: &mut Reader<&[u8]>, e: &BytesStart) -> Text {
        let item = Text {
            text: get_attribute(e, "value").map(|text| unescape(&text).unwrap().into_owned()),
        };
        skip_rest_of_element(reader);
        item
    }

    pub fn display(self) -> Option<String> {
        self.text
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

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
            Text::from_xml(&mut reader, &element).display().unwrap(),
            expected_output
        );
    }
}
