use crate::script_steps::constants::{ScriptStep, id_to_script_step};
use crate::script_steps::parameters::boolean::Boolean;
use crate::script_steps::parameters::dialog_field::DialogField;
use crate::script_steps::parameters::field_reference::FieldReference;
use crate::script_steps::parameters::format::join;
use crate::script_steps::parameters::{
    animation, button, comment, data_source_reference, device, email, files, found_sets, labeled,
    layout_reference, list, objects, options, printing, privilege_set_reference, related,
    script_reference, scripting, target, window_reference,
};
use crate::utils::xml_node::Node;

/// Displays each `<Parameter>` of a step's `<ParameterValues>`, in order.
pub fn display(parameter_values: &Node, step_id: u32) -> Vec<String> {
    parameter_values
        .children_named("Parameter")
        .filter_map(|parameter| display_parameter(parameter, step_id))
        .collect()
}

/// Displays a single `<Parameter>`; this is the one place that maps parameter types
/// to their formatters.
pub(crate) fn display_parameter(parameter: &Node, step_id: u32) -> Option<String> {
    let step = id_to_script_step(step_id);
    let Some(parameter_type) = parameter.attr("type") else {
        // Send Event stores all its options in a single untyped parameter
        return match step {
            ScriptStep::SendEvent => scripting::send_event(parameter),
            _ => None,
        };
    };

    match parameter_type {
        // Simple values
        "Boolean" => Boolean::from_node(parameter, step_id).display(),
        "Calculation" => {
            let calculation = parameter.calculation();
            match step {
                ScriptStep::InstallOnTimerScript => calculation.map(|c| format!("Interval: {c}")),
                _ => calculation,
            }
        }
        "Name" | "Condition" | "ErrorCode" | "ErrorMessage" | "CustomDebugInfo" | "Title"
        | "Message" | "Password" | "Old" | "New" => parameter
            .calculation()
            .map(|d| format!("{parameter_type}: {d}")),
        "Comment" => Some(comment::display(parameter)),
        "List" if step == ScriptStep::InsertFromDevice => device::device_source(parameter),
        "List"
            if matches!(
                step,
                ScriptStep::PerformScriptOnServer | ScriptStep::PerformScriptOnServerWithCallback
            ) =>
        {
            list::display(parameter, step_id).map(|specified| format!("Specified: {specified}"))
        }
        "List" | "AccountType" => list::display(parameter, step_id),

        // References
        "Target" => target::display(parameter).map(|d| format!("Target: {d}")),
        "FieldReference" => Some(FieldReference::from_node(parameter).display()),
        "LayoutReferenceContainer" => layout_reference::display(parameter),
        "ScriptReference" => script_reference::display(parameter),
        "DataSourceReference" => data_source_reference::display(parameter),
        "PrivilegeSetReference" => privilege_set_reference::display(parameter),
        "Variable" => join(scripting::variable(parameter)),

        // Windows, layouts and layout objects
        "Animation" => animation::display(parameter),
        "WindowReference" => Some(window_reference::display(parameter)).filter(|d| !d.is_empty()),
        "Related" => Some(related::display(parameter)).filter(|d| !d.is_empty()),
        "Select" => join(objects::select(parameter)),
        "Object" => join(objects::object(parameter)),
        "Source" => join(objects::source(parameter)),
        "CustomMenuSet" => join(objects::custom_menu_set(parameter)),

        // Show Custom Dialog
        "Button1" | "Button2" | "Button3" => button::display(parameter, parameter_type),
        "Field1" | "Field2" | "Field3" => DialogField::from_node(parameter).display(parameter_type),

        // Files, records and found sets
        "UniversalPathList" => join(files::universal_path_list(parameter)),
        "Export" => join(files::export(parameter)),
        "ImportField" => join(files::import_field(parameter)),
        "Options" => join(options::options(parameter)),
        "Restore" => join(found_sets::restore(parameter)),
        "SortSpecification" => join(found_sets::sort_specification(parameter)),
        "FindRequest" => join(found_sets::find_requests(parameter)),
        "PageSetup" => join(printing::page_setup(parameter)),
        "Print" => join(printing::print(parameter)),

        // Scripting and external services
        "Parameter" => join(scripting::parameter(parameter)),
        "URL" => join(scripting::url(parameter)),
        "SQL" => join(scripting::sql(parameter)),
        "Flow" => join(scripting::flow(parameter)),
        "Email" => join(email::email(parameter)),
        "Voice" => join(device::voice(parameter)),

        // Parameters that wrap a single Calculation, Boolean or List with a fixed label
        other => match labeled::label(other, step_id) {
            Some(label) => labeled::display(parameter, step_id, other, label),
            None => Some(format!(r#"⚠️ PARAMETER "{parameter_type}" NOT PARSED ⚠️"#)),
        },
    }
}

/// Test helper: displays a single `<Parameter>` element.
#[cfg(test)]
pub(crate) fn parse(xml: &str, step_id: u32) -> Option<String> {
    display_parameter(&Node::parse(xml).unwrap(), step_id)
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
