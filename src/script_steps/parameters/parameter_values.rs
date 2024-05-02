use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::script_steps::parameters::boolean::Boolean;
use crate::utils::attributes::get_attribute;

pub struct ParameterValues {
    pub parameters: Vec<String>,
}

impl ParameterValues {
    pub fn from_xml(
        reader: &mut Reader<&[u8]>,
        _e: &BytesStart,
    ) -> Result<ParameterValues, String> {
        let mut depth = 1;
        let mut item = ParameterValues {
            parameters: Vec::new(),
        };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if let b"Parameter" = e.name().as_ref() {
                        let parameter_type = get_attribute(&e, "type").unwrap();
                        match parameter_type.as_str() {
                            "Boolean" => {
                                if let Ok(boolean_param) = Boolean::from_xml(reader, &e) {
                                    if let Some(display) = boolean_param.display() {
                                        item.parameters.push(display);
                                    }
                                }
                            }
                            _ => {
                                eprintln!("Unknown parameter type: {}", parameter_type)
                            }
                        }

                        depth -= 1;
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
        Some(self.parameters.join(" ; "))
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    use crate::script_steps::parameters::parameter_values::ParameterValues;

    #[test]
    fn test() {
        let xml_input = r#"
        <ParameterValues membercount="1">
            <Parameter type="Boolean">
                <Boolean type="Pause" id="16777216" value="False"></Boolean>
            </Parameter>
        </ParameterValues>
        "#;

        let mut reader = Reader::from_str(xml_input.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "Pause: OFF".to_string();
        assert_eq!(
            ParameterValues::from_xml(&mut reader, &element)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
