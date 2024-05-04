use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::script_steps::constants::{id_to_script_step, ScriptStep};
use crate::script_steps::parameters::constants::CommitRecordRequestsOptions;
use crate::utils::attributes::get_attributes;

#[derive(Debug, Default)]
pub struct Boolean {
    pub step_id: u32,
    pub id: Option<u32>,
    pub name: Option<String>,
    pub value: Option<bool>,
}

impl Boolean {
    pub fn from_xml(
        reader: &mut Reader<&[u8]>,
        _e: &BytesStart,
        _step_id: &u32,
    ) -> Result<Boolean, String> {
        let mut depth = 1;
        let mut item = Boolean {
            step_id: *_step_id,
            id: None,
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
                                "id" => {
                                    if let Ok(id) = attr.1.parse::<u32>() {
                                        item.id = Some(id);
                                    }
                                }
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

    pub fn should_drop(&self) -> bool {
        let step_id = id_to_script_step(&self.step_id);
        let param_id = self.id.unwrap_or(0);
        let value = self.value.unwrap_or(false);

        matches!(
            (step_id, param_id, value),
            (
                ScriptStep::CommitRecordRequests,
                CommitRecordRequestsOptions::SKIP_DATA_ENTRY_VALIDATION,
                false,
            ) | (
                ScriptStep::CommitRecordRequests,
                CommitRecordRequestsOptions::OVERRIDE_ESS_LOCKING_CONFLICTS,
                false,
            )
        )
    }

    pub fn should_hide_bool(&self) -> bool {
        let step_id = id_to_script_step(&self.step_id);
        let param_id = self.id.unwrap_or(0);
        let value = self.value.unwrap_or(false);

        matches!(
            (step_id, param_id, value),
            (
                ScriptStep::CommitRecordRequests,
                CommitRecordRequestsOptions::SKIP_DATA_ENTRY_VALIDATION,
                true,
            ) | (
                ScriptStep::CommitRecordRequests,
                CommitRecordRequestsOptions::OVERRIDE_ESS_LOCKING_CONFLICTS,
                true,
            )
        )
    }

    pub fn bool_to_string(bool: bool) -> String {
        match bool {
            true => "ON".to_string(),
            false => "OFF".to_string(),
        }
    }

    pub fn display(&self) -> Option<String> {
        if Self::should_drop(self) {
            return None;
        }

        if Self::should_hide_bool(self) {
            return self.name.clone();
        }

        match &self.name {
            Some(name) => self.value.map(|bool_value| {
                let formatted_string = format!("{}: {}", name, Self::bool_to_string(bool_value));
                formatted_string
            }),
            None => self.value.map(Self::bool_to_string),
        }
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
        let script_id: u32 = 0;

        assert_eq!(
            Boolean::from_xml(&mut reader, &element, &script_id)
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
        let script_id: u32 = 0;
        assert_eq!(
            Boolean::from_xml(&mut reader, &element, &script_id)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
