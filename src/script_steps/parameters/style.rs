use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::utils::attributes::get_attribute;
use crate::utils::xml_utils::skip_rest_of_element;

#[derive(Debug, Default)]
pub struct Style {
    pub style: Option<String>,
}

impl Style {
    pub fn from_xml(reader: &mut Reader<&[u8]>, e: &BytesStart) -> Style {
        let item = Style {
            style: get_attribute(e, "name"),
        };
        skip_rest_of_element(reader);
        item
    }

    pub fn display(self) -> Option<String> {
        self.style.map(|s| format!("Style: {s}"))
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

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
            Style::from_xml(&mut reader, &element).display().unwrap(),
            expected_output
        );
    }
}
