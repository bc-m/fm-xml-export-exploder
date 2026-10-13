use crate::script_steps::constants::{ScriptStep, id_to_script_step};
use crate::script_steps::parameters::boolean::Boolean;
use crate::script_steps::parameters::dialog_field::DialogField;
use crate::script_steps::parameters::field_reference::FieldReference;
use crate::script_steps::parameters::{
    animation, button, comment, data_source_reference, labeled, layout_reference, list,
    privilege_set_reference, related, script_reference, structured, target, window_reference,
};
use crate::utils::xml_node::Node;

/// Displays each `<Parameter>` of a step's `<ParameterValues>`, in order.
pub fn display(parameter_values: &Node, step_id: u32) -> Vec<String> {
    let mut parameters = Vec::new();
    for parameter in parameter_values.children_named("Parameter") {
        let Some(parameter_type) = parameter.attr("type") else {
            // Send Event stores all its options in a single untyped parameter
            if id_to_script_step(step_id) == ScriptStep::SendEvent {
                parameters.extend(structured::send_event(parameter));
            }
            continue;
        };
        let step = id_to_script_step(step_id);
        let parsed = match parameter_type {
            "Animation" => animation::display(parameter),
            "Boolean" => Boolean::from_node(parameter, step_id).display(),
            "List" if step == ScriptStep::InsertFromDevice => structured::device_source(parameter),
            "List"
                if matches!(
                    step,
                    ScriptStep::PerformScriptOnServer
                        | ScriptStep::PerformScriptOnServerWithCallback
                ) =>
            {
                list::display(parameter, step_id).map(|specified| format!("Specified: {specified}"))
            }
            "List" | "AccountType" => list::display(parameter, step_id),
            "Target" => target::display(parameter).map(|d| format!("Target: {d}")),
            "Calculation" => {
                let calculation = parameter.calculation();
                match step {
                    ScriptStep::InstallOnTimerScript => {
                        calculation.map(|c| format!("Interval: {c}"))
                    }
                    _ => calculation,
                }
            }
            "Name" | "Condition" | "ErrorCode" | "ErrorMessage" | "CustomDebugInfo" | "Title"
            | "Message" | "Password" | "Old" | "New" => parameter
                .calculation()
                .map(|d| format!("{parameter_type}: {d}")),
            "LayoutReferenceContainer" => layout_reference::display(parameter),
            "FieldReference" => Some(FieldReference::from_node(parameter).display()),
            "Comment" => Some(comment::display(parameter)),
            "WindowReference" => {
                Some(window_reference::display(parameter)).filter(|d| !d.is_empty())
            }
            "Related" => Some(related::display(parameter)).filter(|d| !d.is_empty()),
            "ScriptReference" => script_reference::display(parameter),
            "DataSourceReference" => data_source_reference::display(parameter),
            "PrivilegeSetReference" => privilege_set_reference::display(parameter),
            "Button1" | "Button2" | "Button3" => button::display(parameter, parameter_type),
            "Field1" | "Field2" | "Field3" => {
                DialogField::from_node(parameter).display(parameter_type)
            }
            other if structured::is_structured(other) => structured::display(parameter, other),
            other => match labeled::label(other, step_id) {
                Some(label) => labeled::display(parameter, step_id, other, label),
                None => Some(format!(r#"⚠️ PARAMETER "{parameter_type}" NOT PARSED ⚠️"#)),
            },
        };
        parameters.extend(parsed);
    }
    parameters
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::parameter_values;

    #[test]
    fn test() {
        let xml = r#"
            <ParameterValues membercount="1">
                <Parameter type="Boolean">
                    <Boolean type="Pause" id="16777216" value="False"></Boolean>
                </Parameter>
            </ParameterValues>
        "#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "Pause: OFF";
        let parameters = parameter_values::display(&element, 0);
        assert_eq!(parameters.join(" ; "), expected_output);
    }
}
