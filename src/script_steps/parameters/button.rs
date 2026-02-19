use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::utils::attributes::get_attribute;
use crate::utils::xml_utils::cdata_to_string;

#[derive(Debug, Default)]
pub struct Button {
    pub label: Option<String>,
    pub commit: bool,
}

impl Button {
    pub fn from_xml(reader: &mut Reader<&[u8]>, e: &BytesStart) -> Button {
        let mut label = get_attribute(e, "value");
        let mut commit = false;
        let mut in_text = false;
        let mut depth = 1;

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(inner)) => {
                    depth += 1;
                    match inner.name().as_ref() {
                        b"Text" => in_text = true,
                        b"Boolean" => {
                            if let Some(val) = get_attribute(&inner, "value") {
                                commit = val == "True";
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::CData(cdata)) => {
                    if in_text && label.is_none() {
                        let text = cdata_to_string(&cdata);
                        if !text.is_empty() {
                            label = Some(text);
                        }
                    }
                }
                Ok(Event::End(end)) => {
                    if end.name().as_ref() == b"Text" {
                        in_text = false;
                    }
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            buf.clear();
        }

        Button { label, commit }
    }

    pub fn display(&self, button_type: &str) -> Option<String> {
        let label = self.label.as_ref()?;
        if label.is_empty() {
            return None;
        }

        let name = match button_type {
            "Button1" => "Default Button",
            "Button2" => "Button 2",
            "Button3" => "Button 3",
            _ => button_type,
        };

        Some(format!("{name}: {label}"))
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    use super::Button;

    #[test]
    fn test_button_with_calculation() {
        let xml = r#"
            <Parameter type="Button1">
                <Calculation datatype="1" position="5">
                    <Calculation>
                        <Text><![CDATA["OK"]]></Text>
                    </Calculation>
                </Calculation>
                <Boolean type="Commit" value="False"></Boolean>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let button = Button::from_xml(&mut reader, &element);
        assert_eq!(button.label, Some(r#""OK""#.to_string()));
        assert!(!button.commit);
        assert_eq!(
            button.display("Button1"),
            Some(r#"Default Button: "OK""#.to_string())
        );
    }

    #[test]
    fn test_button_with_value_attribute() {
        let xml = r#"
            <Parameter type="Button1" value="Save">
                <Boolean type="Commit" value="True"></Boolean>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let button = Button::from_xml(&mut reader, &element);
        assert_eq!(button.label, Some("Save".to_string()));
        assert!(button.commit);
        assert_eq!(
            button.display("Button2"),
            Some("Button 2: Save".to_string())
        );
    }

    #[test]
    fn test_empty_button() {
        let xml = r#"
            <Parameter type="Button2">
                <Boolean type="Commit" value="False"></Boolean>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let button = Button::from_xml(&mut reader, &element);
        assert_eq!(button.label, None);
        assert!(!button.commit);
        assert_eq!(button.display("Button2"), None);
    }

    #[test]
    fn test_button3_display() {
        let xml = r#"
            <Parameter type="Button3" value="Maybe">
                <Boolean type="Commit" value="False"></Boolean>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let button = Button::from_xml(&mut reader, &element);
        assert_eq!(
            button.display("Button3"),
            Some("Button 3: Maybe".to_string())
        );
    }
}
