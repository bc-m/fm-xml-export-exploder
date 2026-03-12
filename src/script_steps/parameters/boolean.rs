use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::script_steps::constants::{ScriptStep, id_to_script_step};
use crate::script_steps::parameters::constants::{
    commit_record_requests, go_to_field, refresh_window,
};
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
        _: &BytesStart,
        step_id: u32,
    ) -> Result<Boolean, String> {
        let mut depth = 1;
        let mut item = Boolean {
            step_id,
            ..Default::default()
        };

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if let b"Boolean" = e.name().as_ref() {
                        for attr in get_attributes(&e) {
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

    fn should_hide_bool(&self) -> bool {
        let step_id = id_to_script_step(self.step_id);
        let param_id = self.id.unwrap_or(0);

        matches!(
            (step_id, param_id),
            (
                ScriptStep::CommitRecordRequests,
                commit_record_requests::SKIP_DATA_ENTRY_VALIDATION
            ) | (
                ScriptStep::CommitRecordRequests,
                commit_record_requests::OVERRIDE_ESS_LOCKING_CONFLICTS
            ) | (
                ScriptStep::RefreshWindow,
                refresh_window::FLUSH_CACHED_JOIN_RESULTS
            ) | (
                ScriptStep::RefreshWindow,
                refresh_window::FLUSH_CACHED_EXTERNAL_DATA
            ) | (ScriptStep::GoToField, go_to_field::SELECT_PERFORM)
        )
    }

    pub fn on_off(value: bool) -> &'static str {
        if value { "ON" } else { "OFF" }
    }

    pub fn display(self) -> Option<String> {
        if self.should_hide_bool() {
            return if self.value.unwrap_or(false) {
                self.name
            } else {
                None
            };
        }

        match self.name {
            Some(name) => self.value.map(|v| format!("{}: {}", name, Self::on_off(v))),
            None => self.value.map(|v| Self::on_off(v).to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

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
            Boolean::from_xml(&mut reader, &element, 0)
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
            Boolean::from_xml(&mut reader, &element, 0)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
