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
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart, step_id: u32) -> ParameterValues {
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
                    let parsed = match parameter_type.as_str() {
                        "Animation" => Animation::from_xml(reader, &e).display(),
                        "Boolean" => Boolean::from_xml(reader, &e, step_id).display(),
                        "List" => List::from_xml(reader, &e, step_id).display(),
                        "Target" => Target::from_xml(reader, &e)
                            .display()
                            .map(|d| format!("Target: {d}")),
                        "Calculation" => Calculation::from_xml(reader, &e).display(),
                        "Name" | "Condition" | "ErrorCode" | "ErrorMessage" | "CustomDebugInfo"
                        | "Title" | "Message" => Calculation::from_xml(reader, &e)
                            .display()
                            .map(|d| format!("{parameter_type}: {d}")),
                        "LayoutReferenceContainer" => {
                            LayoutReferenceContainer::from_xml(reader, &e).display()
                        }
                        "FieldReference" => Some(FieldReference::from_xml(reader, &e).display()),
                        "Comment" => Some(Comment::from_xml(reader, &e)),
                        "WindowReference" => {
                            let d = WindowReference::from_xml(reader, &e).display();
                            if d.is_empty() { None } else { Some(d) }
                        }
                        "Related" => {
                            let d = Related::from_xml(reader, &e).display();
                            if d.is_empty() { None } else { Some(d) }
                        }
                        "ScriptReference" => ScriptReference::from_xml(reader, &e).display(),
                        "DataSourceReference" => {
                            DataSourceReference::from_xml(reader, &e).display()
                        }
                        "Button1" | "Button2" | "Button3" => {
                            Button::from_xml(reader, &e).display(&parameter_type)
                        }
                        "Field1" | "Field2" | "Field3" => {
                            DialogField::from_xml(reader, &e).display(&parameter_type)
                        }
                        _ => {
                            item.parameters
                                .push(format!(r#"⚠️ PARAMETER "{parameter_type}" NOT PARSED ⚠️"#));
                            continue;
                        }
                    };
                    depth -= 1;
                    if let Some(display) = parsed {
                        item.parameters.push(display);
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

        item
    }

    pub fn display(self) -> String {
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

                params.join(" ; ")
            }
            ScriptStep::SetErrorLogging => {
                let mut iter = self.parameters.into_iter();
                let on_off = match iter.next() {
                    Some(first) if first.ends_with(": ON") => "ON".to_string(),
                    _ => "OFF".to_string(),
                };
                match iter.next() {
                    Some(second) => format!("{on_off} ; {second}"),
                    None => on_off,
                }
            }
            _ => self.parameters.join(" ; "),
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

        let expected_output = "Pause: OFF";
        assert_eq!(
            ParameterValues::from_xml(&mut reader, &element, 0).display(),
            expected_output
        );
    }
}
