use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::script_steps::constants::{id_to_script_step, ScriptStep};
use crate::script_steps::parameters::boolean::Boolean;
use crate::script_steps::parameters::calculation::Calculation;
use crate::script_steps::parameters::comment::Comment;
use crate::script_steps::parameters::field_reference::FieldReference;
use crate::script_steps::parameters::layout_reference::LayoutReferenceContainer;
use crate::script_steps::parameters::list::List;
use crate::script_steps::parameters::target::Target;
use crate::utils::attributes::get_attribute;

pub struct ParameterValues {
    pub step_id: u32,
    pub parameters: Vec<String>,
}

impl ParameterValues {
    pub fn from_xml(
        reader: &mut Reader<&[u8]>,
        _e: &BytesStart,
        step_id: &u32,
    ) -> Result<ParameterValues, String> {
        let mut depth = 1;
        let mut item = ParameterValues {
            step_id: *step_id,
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
                        let parameter_type = get_attribute(&e, "type");
                        if parameter_type.is_none() {
                            continue;
                        }

                        let parameter_type = parameter_type.unwrap();
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
                            "Target" => {
                                if let Ok(param_value) = Target::from_xml(reader, &e) {
                                    if let Some(display) = param_value.display() {
                                        item.parameters.push(format!(
                                            "{}: {}",
                                            parameter_type.as_str(),
                                            display
                                        ));
                                    }
                                }
                                depth -= 1;
                            }
                            "Calculation" => {
                                if let Ok(param_value) = Calculation::from_xml(reader, &e) {
                                    item.parameters.push(param_value);
                                }
                                depth -= 1;
                            }
                            "Condition" | "ErrorCode" | "ErrorMessage" | "CustomDebugInfo" => {
                                if let Ok(param_value) = Calculation::from_xml(reader, &e) {
                                    item.parameters.push(format!(
                                        "{}: {}",
                                        parameter_type.as_str(),
                                        param_value
                                    ));
                                }
                                depth -= 1;
                            }
                            "LayoutReferenceContainer" => {
                                if let Ok(param_value) =
                                    LayoutReferenceContainer::from_xml(reader, &e)
                                {
                                    if let Some(display) = param_value.display() {
                                        item.parameters.push(display);
                                    }
                                }
                            }
                            "FieldReference" => {
                                if let Ok(param_value) = FieldReference::from_xml(reader, &e) {
                                    if let Some(display) = param_value.display() {
                                        item.parameters.push(display);
                                    }
                                }
                            }
                            "Comment" => {
                                if let Ok(param_value) = Comment::from_xml(reader, &e) {
                                    item.parameters.push(param_value);
                                }
                                depth -= 1;
                            }
                            _ => {
                                item.parameters.push(format!(
                                    r#"⚠️ PARAMETER "{}" NOT PARSED ⚠️"#,
                                    parameter_type
                                ));
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
        match id_to_script_step(&self.step_id) {
            ScriptStep::RevertTransaction => {
                let mut modified_parameters = self.parameters.clone();
                modified_parameters
                    .retain(|param| !param.ends_with(": ON") && !param.ends_with(": OFF"));

                let mut iter = modified_parameters.iter().rev();
                if let Some(last) = iter.next() {
                    if last.starts_with("ErrorMessage") {
                        if let Some(second_last) = iter.next() {
                            if !second_last.starts_with("ErrorCode") {
                                modified_parameters.pop();
                            }
                        }
                    }
                }

                Some(modified_parameters.join(" ; "))
            }
            ScriptStep::SetErrorLogging => {
                let mut modified_parameters: Vec<String> = Vec::new();

                let mut iter = self.parameters.iter();
                if let Some(first) = iter.next() {
                    if first.ends_with(": ON") {
                        modified_parameters.push(String::from("ON"))
                    } else {
                        modified_parameters.push(String::from("OFF"))
                    }
                }
                if let Some(second) = iter.next() {
                    modified_parameters.push(second.clone());
                }

                Some(modified_parameters.join(" ; "))
            }
            _ => Some(self.parameters.join(" ; ")),
        }
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
        let script_id: u32 = 0;
        assert_eq!(
            ParameterValues::from_xml(&mut reader, &element, &script_id)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
