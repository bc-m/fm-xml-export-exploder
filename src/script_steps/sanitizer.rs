use crate::script_steps;
use crate::script_steps::constants::{id_to_script_step, ScriptStep};

pub fn sanitize(step_id: &str, step_xml: &str) -> Option<String> {
    let is_enabled = script_steps::is_enabled::sanitize(step_xml);

    let step_sanitized = match id_to_script_step(step_id) {
        ScriptStep::PerformScript => script_steps::perform_script::sanitize(step_xml),
        ScriptStep::GoToLayout => script_steps::go_to_layout::sanitize(step_xml),
        ScriptStep::GoToRecordRequestPage => script_steps::go_to_record::sanitize(step_xml),
        ScriptStep::GoToField => script_steps::go_to_field::sanitize(step_xml),
        ScriptStep::OmitMultipleRecords => script_steps::omit_multiple_records::sanitize(step_xml),
        ScriptStep::PerformFind => script_steps::perform_find::sanitize(step_xml),
        ScriptStep::InsertText => script_steps::insert_text::sanitize(step_xml),
        ScriptStep::IfStart => script_steps::if_start::sanitize(step_xml),
        ScriptStep::ExitLoopIf => script_steps::if_start::sanitize(step_xml),
        ScriptStep::CommitRecordRequests => script_steps::commit::sanitize(step_xml),
        ScriptStep::SetField => script_steps::set_field_data::sanitize(step_xml),
        ScriptStep::Comment => script_steps::comment::sanitize(step_xml),
        ScriptStep::ReplaceFieldContents => {
            script_steps::replace_field_contents::sanitize(step_xml)
        }
        ScriptStep::GoToPortalRow => script_steps::go_to_portal_row::sanitize(step_xml),
        ScriptStep::ExitScript => script_steps::exit_script::sanitize(step_xml),
        ScriptStep::CloseWindow => script_steps::close_window::sanitize(step_xml),
        ScriptStep::NewWindow => script_steps::new_window::sanitize(step_xml),
        ScriptStep::IfElse => script_steps::if_start::sanitize(step_xml),
        ScriptStep::ConstrainFoundSet => script_steps::perform_find::sanitize(step_xml),
        ScriptStep::ExtendFoundSet => script_steps::perform_find::sanitize(step_xml),
        ScriptStep::SetVariable => script_steps::set_variable::sanitize(step_xml),
        ScriptStep::GoToObject => script_steps::go_to_object::sanitize(step_xml),
        ScriptStep::RefreshObject => script_steps::refresh_object::sanitize(step_xml),
        ScriptStep::Unknown => Option::from(format!(
            "{} ⚠️⚠️⚠️ FM-XML-EXPORT-EXPLODER: UNKNOWN SCRIPT-STEP [ ID: {:?} ] ⚠️⚠️⚠️",
            script_steps::primitive::sanitize(step_xml).unwrap(),
            step_id
        )),
        _ => script_steps::sanitize::from_xml(step_id, step_xml),
    };

    match step_sanitized {
        None => {
            println!("Could not parse: {}", step_xml);
            None
        }
        Some(step) => match is_enabled {
            true => Some(step),
            false => Some(format!("// {}", step)),
        },
    }
}
