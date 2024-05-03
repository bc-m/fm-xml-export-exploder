use crate::script_steps::constants::id_to_script_step;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::script_steps::parameters::boolean::Boolean;
use crate::script_steps::parameters::list::List;
use crate::utils::attributes::get_attribute;

pub struct ParameterValues {
    pub parameters: Vec<String>,
}

impl ParameterValues {
    pub fn from_xml(
        reader: &mut Reader<&[u8]>,
        _e: &BytesStart,
        step_id: &str,
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
                                if let Ok(param_value) = Boolean::from_xml(reader, &e, step_id) {
                                    if let Some(display) = param_value.display() {
                                        item.parameters.push(display);
                                    }
                                }
                                depth -= 1;
                            }
                            "List" => {
                                if let Ok(param_value) = List::from_xml(reader, &e, step_id) {
                                    if let Some(display) = param_value.display() {
                                        item.parameters.push(display);
                                    }
                                }
                                depth -= 1;
                            }
                            _ => {
                                eprintln!(
                                    "Unknown parameter \"{}\" in step \"{}\"",
                                    parameter_type,
                                    id_to_script_step(step_id)
                                )
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
        let xml = r#"
            <ParameterValues membercount="1">
                <Parameter type="Boolean">
                    <Boolean type="Pause" id="16777216" value="False"></Boolean>
                </Parameter>
            </ParameterValues>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "Pause: OFF".to_string();
        assert_eq!(
            ParameterValues::from_xml(&mut reader, &element, "")
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
