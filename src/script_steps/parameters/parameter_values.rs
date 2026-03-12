use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::script_steps::constants::{ScriptStep, id_to_script_step};
use crate::script_steps::parameters::animation::Animation;
use crate::script_steps::parameters::boolean::Boolean;
use crate::script_steps::parameters::button::Button;
use crate::script_steps::parameters::calculation::Calculation;
use crate::script_steps::parameters::comment::Comment;
use crate::script_steps::parameters::data_source_reference::DataSourceReference;
use crate::script_steps::parameters::dialog_field::DialogField;
use crate::script_steps::parameters::field_reference::FieldReference;
use crate::script_steps::parameters::layout_reference::LayoutReferenceContainer;
use crate::script_steps::parameters::list::List;
use crate::script_steps::parameters::related::Related;
use crate::script_steps::parameters::script_reference::ScriptReference;
use crate::script_steps::parameters::target::Target;
use crate::script_steps::parameters::window_reference::WindowReference;
use crate::utils::attributes::get_attribute;

pub struct ParameterValues {
    pub step_id: u32,
    pub parameters: Vec<String>,
}

impl ParameterValues {
    pub fn from_xml(
        reader: &mut Reader<&[u8]>,
        _: &BytesStart,
        step_id: u32,
    ) -> Result<ParameterValues, String> {
        let mut depth = 1;
        let mut item = ParameterValues {
            step_id,
            parameters: Vec::new(),
        };

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if depth != 2 || e.name().as_ref() != b"Parameter" {
                        continue;
                    }

                    let Some(parameter_type) = get_attribute(&e, "type") else {
                        continue;
                    };
                    match parameter_type.as_str() {
                        "Animation" => {
                            if let Ok(param_value) = Animation::from_xml(reader, &e)
                                && let Some(display) = param_value.display()
                            {
                                item.parameters.push(display);
                            }
                            depth -= 1;
                        }
                        "Boolean" => {
                            if let Ok(param_value) = Boolean::from_xml(reader, &e, step_id)
                                && let Some(display) = param_value.display()
                            {
                                item.parameters.push(display);
                            }
                            depth -= 1;
                        }
                        "List" => {
                            if let Ok(param_value) = List::from_xml(reader, &e, step_id)
                                && let Some(display) = param_value.display()
                            {
                                item.parameters.push(display);
                            }
                            depth -= 1;
                        }
                        "Target" => {
                            if let Ok(param_value) = Target::from_xml(reader, &e)
                                && let Some(display) = param_value.display()
                            {
                                item.parameters.push(format!(
                                    "{}: {}",
                                    parameter_type.as_str(),
                                    display
                                ));
                            }
                            depth -= 1;
                        }
                        "Calculation" => {
                            if let Ok(param_value) = Calculation::from_xml(reader, &e)
                                && let Some(display) = param_value.display()
                            {
                                item.parameters.push(display);
                            }
                            depth -= 1;
                        }
                        "Name" | "Condition" | "ErrorCode" | "ErrorMessage" | "CustomDebugInfo"
                        | "Title" | "Message" => {
                            if let Ok(param_value) = Calculation::from_xml(reader, &e)
                                && let Some(display) = param_value.display()
                            {
                                item.parameters.push(format!(
                                    "{}: {}",
                                    parameter_type.as_str(),
                                    display
                                ));
                            }
                            depth -= 1;
                        }
                        "LayoutReferenceContainer" => {
                            if let Ok(param_value) = LayoutReferenceContainer::from_xml(reader, &e)
                                && let Some(display) = param_value.display()
                            {
                                item.parameters.push(display);
                            }
                            depth -= 1;
                        }
                        "FieldReference" => {
                            if let Ok(param_value) = FieldReference::from_xml(reader, &e)
                                && let Some(display) = param_value.display()
                            {
                                item.parameters.push(display);
                            }
                            depth -= 1;
                        }
                        "Comment" => {
                            if let Ok(param_value) = Comment::from_xml(reader, &e) {
                                item.parameters.push(param_value);
                            }
                            depth -= 1;
                        }
                        "WindowReference" => {
                            if let Some(param_value) = WindowReference::from_xml(reader, &e)
                                && let Some(display) = param_value.display()
                            {
                                item.parameters.push(display);
                            }
                            depth -= 1;
                        }
                        "Related" => {
                            if let Some(param_value) = Related::from_xml(reader, &e)
                                && let Some(display) = param_value.display()
                            {
                                item.parameters.push(display);
                            }
                            depth -= 1;
                        }
                        "ScriptReference" => {
                            if let Some(param_value) = ScriptReference::from_xml(reader, &e)
                                && let Some(display) = param_value.display()
                            {
                                item.parameters.push(display);
                            }
                            depth -= 1;
                        }
                        "DataSourceReference" => {
                            if let Some(param_value) = DataSourceReference::from_xml(reader, &e)
                                && let Some(display) = param_value.display()
                            {
                                item.parameters.push(display);
                            }
                            depth -= 1;
                        }
                        "Button1" | "Button2" | "Button3" => {
                            let button = Button::from_xml(reader, &e);
                            if let Some(display) = button.display(parameter_type.as_str()) {
                                item.parameters.push(display);
                            }
                            depth -= 1;
                        }
                        "Field1" | "Field2" | "Field3" => {
                            let field = DialogField::from_xml(reader, &e);
                            if let Some(display) = field.display(parameter_type.as_str()) {
                                item.parameters.push(display);
                            }
                            depth -= 1;
                        }
                        _ => {
                            item.parameters
                                .push(format!(r#"⚠️ PARAMETER "{parameter_type}" NOT PARSED ⚠️"#));
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

    pub fn display(self) -> Option<String> {
        match id_to_script_step(self.step_id) {
            ScriptStep::RevertTransaction => {
                let mut params: Vec<_> = self
                    .parameters
                    .into_iter()
                    .filter(|p| !p.ends_with(": ON") && !p.ends_with(": OFF"))
                    .collect();

                // Remove trailing ErrorMessage if not preceded by ErrorCode
                let len = params.len();
                if len >= 2
                    && params[len - 1].starts_with("ErrorMessage")
                    && !params[len - 2].starts_with("ErrorCode")
                {
                    params.pop();
                }

                Some(params.join(" ; "))
            }
            ScriptStep::SetErrorLogging => {
                let mut result = Vec::new();
                let mut iter = self.parameters.into_iter();

                if let Some(first) = iter.next() {
                    let on_off = if first.ends_with(": ON") { "ON" } else { "OFF" };
                    result.push(on_off.to_string());
                }
                if let Some(second) = iter.next() {
                    result.push(second);
                }

                Some(result.join(" ; "))
            }
            _ => Some(self.parameters.join(" ; ")),
        }
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

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
            ParameterValues::from_xml(&mut reader, &element, 0)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
