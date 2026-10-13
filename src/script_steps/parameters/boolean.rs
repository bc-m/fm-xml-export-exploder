use crate::script_steps::constants::{ScriptStep, id_to_script_step};
use crate::script_steps::parameters::constants::{
    commit_record_requests, go_to_field, refresh_window,
};
use crate::utils::xml_node::Node;

#[derive(Debug, Default)]
pub struct Boolean {
    pub step_id: u32,
    pub id: Option<u32>,
    pub name: Option<String>,
    pub value: Option<bool>,
}

impl Boolean {
    pub fn from_node(parameter: &Node, step_id: u32) -> Boolean {
        let mut item = Boolean {
            step_id,
            ..Default::default()
        };
        for boolean in parameter
            .descendants()
            .into_iter()
            .filter(|n| n.name == "Boolean")
        {
            for (key, value) in &boolean.attributes {
                match key.as_str() {
                    "id" => item.id = value.parse().ok(),
                    "type" => item.name = Some(value.clone()),
                    "value" => match value.as_str() {
                        "True" => item.value = Some(true),
                        "False" => item.value = Some(false),
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        item
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
        // Collapsed is a UI-only flag (Script Workspace visual state) — never shown in output
        if self.name.as_deref() == Some("Collapsed") {
            return None;
        }

        if self.should_hide_bool() {
            // Hidden booleans only show their name when the value is true
            return self.value.filter(|&v| v).and(self.name);
        }

        let value = self.value?;
        let on_off = Self::on_off(value);
        Some(match self.name {
            Some(name) => format!("{name}: {on_off}"),
            None => on_off.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::boolean::Boolean;

    #[test]
    fn test() {
        let xml = r#"
            <Parameter type="Boolean">
                <Boolean type="Pause" id="16777216" value="False"></Boolean>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "Pause: OFF".to_string();
        assert_eq!(
            Boolean::from_node(&element, 0).display().unwrap(),
            expected_output
        );
    }

    #[test]
    fn test_collapsed_returns_none() {
        let xml = r#"
            <Parameter type="Boolean">
                <Boolean type="Collapsed" id="0" value="False"></Boolean>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();

        assert_eq!(
            Boolean::from_node(&element, 0).display(),
            None,
            "Collapsed boolean should produce no output"
        );
    }

    #[test]
    fn test_without_name() {
        let xml = r#"
            <Parameter type="Boolean">
                <Boolean value="False"></Boolean>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "OFF".to_string();
        assert_eq!(
            Boolean::from_node(&element, 0).display().unwrap(),
            expected_output
        );
    }
}
