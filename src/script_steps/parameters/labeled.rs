use crate::script_steps::constants::{ScriptStep, id_to_script_step};
use crate::script_steps::parameters::boolean::Boolean;
use crate::script_steps::parameters::list;
use crate::utils::xml_node::Node;

/// Returns the label FileMaker shows for parameters that wrap a single
/// `<Calculation>`, `<Boolean>` or `<List>` element.
pub fn label(parameter_type: &str, step_id: u32) -> Option<&'static str> {
    Some(match parameter_type {
        "id" => "File ID",
        "size" => "Amount (bytes)",
        "position" if id_to_script_step(step_id) == ScriptStep::SetDataFilePosition => {
            "New position"
        }
        "position" => "Position",
        "UUID" => "UUID",
        "Major" => "Major",
        "Minor" => "Minor",
        "Latitude" => "Latitude",
        "Longitude" => "Longitude",
        "Radius" => "Radius",
        "End" => "End",
        "Volume" => "Volume",
        "Location" => "Default location",
        "FunctionRef" => "Function Name",
        "Timeout" => "Timeout",
        "ReadMultiple" => "Continuous",
        "name" => "Name",
        "find" => "Find",
        "replace" => "Replace",
        "operation" => "Operation",
        "Monitor" => "Monitor",
        "Presentation" => "Presentation",
        "action" => "Action",
        "perform" => "Perform",
        "direction" => "Direction",
        "across" => "Across",
        "within" => "Within",
        "Zoom" => "Zoom",
        "Sequence" => "Sequence",
        "Hide" => "Hide",
        "DisableInteraction" => "Disable interaction",
        "DisableExternalControls" => "Disable external controls",
        "PauseInBackground" => "Pause in background",
        "Start" => "Start",
        "Wait" => "Wait",
        "case" => "Match case",
        "whole" => "Match whole words only",
        "JSONOutput" => "JSON output",
        "CallbackScriptState" => "Callback script state",
        "CallbackScriptParameter" => "Callback script parameter",
        "LLMAction" | "LLMTrainActions" | "RAGAction" => "Action",
        "TemplateType" => "Template type",
        "TableOccurrences" => "Tables",
        "RAGDataSource" | "FineTuneDataSource" => "Data source",
        "Query" => "Query",
        "ModelProvider" => "Model provider",
        "LLMTrainAlgorithm" => "Algorithm",
        "LLMTablesSelection" => "Tables",
        "LLMOptionsSelection" => "Options",
        // Lists that need no label
        "Records" => "",
        _ => return None,
    })
}

/// Displays a parameter whose value is a single wrapped `<Calculation>`, `<Boolean>`
/// or `<List>` element (or plain text) as `label: value`.
pub fn display(
    parameter: &Node,
    step_id: u32,
    parameter_type: &str,
    label: &str,
) -> Option<String> {
    match parameter.children.first().map(|child| child.name.as_str()) {
        Some("Calculation") => parameter
            .calculation()
            .map(|value| format!("{label}: {value}")),
        Some("Boolean") => {
            let mut boolean = Boolean::from_node(parameter, step_id);
            // Keep descriptive types like "Start immediately", replace internal ones like "DisableInteraction"
            if boolean
                .name
                .as_deref()
                .is_none_or(|name| name == parameter_type)
            {
                boolean.name = Some(label.to_string());
            }
            boolean.display()
        }
        Some("List") => list::display(parameter, step_id)
            .filter(|value| !value.is_empty())
            .map(|value| with_label(label, value)),
        Some(_) => None,
        None => Some(parameter.text())
            .filter(|text| !text.is_empty())
            .map(|text| with_label(label, text.to_string())),
    }
}

fn with_label(label: &str, value: String) -> String {
    if label.is_empty() {
        value
    } else {
        format!("{label}: {value}")
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use super::*;

    fn parse(xml: &str, step_id: u32) -> Option<String> {
        let e = Node::parse(xml).unwrap();
        let parameter_type = e.attr("type").unwrap().to_string();
        let label = label(&parameter_type, step_id).unwrap();
        display(&e, step_id, &parameter_type, label)
    }

    #[test]
    fn calculation() {
        let xml = r#"<Parameter type="size"><Calculation datatype="2" position="1"><Calculation><Text><![CDATA[123]]></Text></Calculation></Calculation></Parameter>"#;
        assert_eq!(parse(xml, 193).as_deref(), Some("Amount (bytes): 123"));
    }

    #[test]
    fn position_depends_on_step() {
        let xml = r#"<Parameter type="position"><Calculation datatype="2" position="1"><Calculation><Text><![CDATA[$Pos]]></Text></Calculation></Calculation></Parameter>"#;
        assert_eq!(parse(xml, 195).as_deref(), Some("New position: $Pos"));
        assert_eq!(parse(xml, 177).as_deref(), Some("Position: $Pos"));
    }

    #[test]
    fn list() {
        let xml = r#"<Parameter type="Monitor"><List name="iBeacon" value="0"></List></Parameter>"#;
        assert_eq!(parse(xml, 185).as_deref(), Some("Monitor: iBeacon"));
    }

    #[test]
    fn boolean_with_label() {
        let xml = r#"<Parameter type="DisableInteraction"><Boolean type="DisableInteraction" value="True"></Boolean></Parameter>"#;
        assert_eq!(parse(xml, 177).as_deref(), Some("Disable interaction: ON"));
    }

    #[test]
    fn boolean_keeps_own_type() {
        let xml = r#"<Parameter type="case"><Boolean type="Match case" value="False"></Boolean></Parameter>"#;
        assert_eq!(parse(xml, 128).as_deref(), Some("Match case: OFF"));
    }

    #[test]
    fn same_type_as_boolean_or_calculation() {
        let boolean = r#"<Parameter type="Start"><Boolean value="True" type="Start immediately"></Boolean></Parameter>"#;
        assert_eq!(
            parse(boolean, 161).as_deref(),
            Some("Start immediately: ON")
        );

        let calculation = r#"<Parameter type="Start"><Calculation datatype="2" position="1"><Calculation><Text><![CDATA[1]]></Text></Calculation></Calculation></Parameter>"#;
        assert_eq!(parse(calculation, 177).as_deref(), Some("Start: 1"));
    }

    #[test]
    fn text() {
        let xml = r#"<Parameter type="operation">uninstall</Parameter>"#;
        assert_eq!(parse(xml, 202).as_deref(), Some("Operation: uninstall"));
    }

    #[test]
    fn list_without_label() {
        let xml = r#"<Parameter type="Records"><List name="First" value="1"></List></Parameter>"#;
        assert_eq!(parse(xml, 218).as_deref(), Some("First"));
    }

    #[test]
    fn empty_parameter() {
        let xml = r#"<Parameter type="find"></Parameter>"#;
        let e = Node::parse(xml).unwrap();
        assert_eq!(display(&e, 128, "find", "Find"), None);
    }
}
