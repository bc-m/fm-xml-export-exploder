use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::utils::attributes::get_attributes;

#[derive(Debug, Default)]
pub struct Boolean {
    pub name: Option<String>,
    pub value: Option<bool>,
}

impl Boolean {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _e: &BytesStart) -> Result<Boolean, String> {
        let mut depth = 1;
        let mut item = Boolean {
            name: None,
            value: None,
        };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if let b"Boolean" = e.name().as_ref() {
                        for attr in get_attributes(&e).unwrap() {
                            match attr.0.as_str() {
                                "type" => item.name = Some(attr.1),
                                "value" => match attr.1.as_str() {
                                    "True" => item.value = Some(true),
                                    "False" => item.value = Some(false),
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                    }
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
        let name = match &self.name {
            Some(name) => name,
            None => {
                return match self.value {
                    None => None,
                    Some(bool) => {
                        return Some(match bool {
                            true => "ON".to_string(),
                            false => "OFF".to_string(),
                        });
                    }
                };
            }
        };

        self.value.map(|bool| {
            format!(
                "{}: {}",
                name,
                match bool {
                    true => "ON",
                    false => "OFF",
                }
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    use crate::script_steps::parameters::boolean::Boolean;

    #[test]
    fn test() {
        let xml = r#"
            <Parameter type="Boolean">
                <Boolean type="Pause" id="16777216" value="False"></Boolean>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "Pause: OFF".to_string();
        assert_eq!(
            Boolean::from_xml(&mut reader, &element)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }

    #[test]
    fn test_without_name() {
        let xml = r#"
            <Parameter type="Boolean">
                <Boolean value="False"></Boolean>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "OFF".to_string();
        assert_eq!(
            Boolean::from_xml(&mut reader, &element)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
