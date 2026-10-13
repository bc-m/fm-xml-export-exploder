use crate::config::Flags;
use crate::script_steps;
use crate::script_steps::constants::{ScriptStep, id_to_script_step};
use crate::utils::xml_node::Node;

pub fn sanitize(step_id: u32, step_xml: &str, flags: &Flags) -> Option<String> {
    let Some(step_node) = Node::parse(step_xml) else {
        eprintln!("Could not parse: {step_xml}");
        return None;
    };
    let step = &step_node;
    let is_enabled = step.attr("enable") != Some("False");
    let obfuscate = flags.obfuscate_passwords;

    let step_sanitized = match id_to_script_step(step_id) {
        ScriptStep::PerformScript => script_steps::perform_script::sanitize(step),
        ScriptStep::GoToRecordRequestPage => script_steps::go_to_record::sanitize(step),
        ScriptStep::InsertText => script_steps::insert_text::sanitize(step),
        ScriptStep::InsertFromURL => script_steps::insert_from_url::sanitize(step),
        ScriptStep::SetField => script_steps::set_field_data::sanitize(step),
        ScriptStep::ReplaceFieldContents => script_steps::replace_field_contents::sanitize(step),
        ScriptStep::GoToPortalRow => script_steps::go_to_portal_row::sanitize(step),
        ScriptStep::ExitScript => script_steps::exit_script::sanitize(step),
        ScriptStep::CloseWindow => script_steps::close_window::sanitize(step),
        ScriptStep::SetVariable => script_steps::set_variable::sanitize(step),
        ScriptStep::GoToObject => script_steps::go_to_object::sanitize(step),
        ScriptStep::RefreshObject => script_steps::refresh_object::sanitize(step),
        ScriptStep::AddAccount => script_steps::accounts::sanitize_add_account(step, obfuscate),
        ScriptStep::ChangePassword => {
            script_steps::accounts::sanitize_change_password(step, obfuscate)
        }
        ScriptStep::DeleteAccount => script_steps::accounts::sanitize_delete_account(step),
        ScriptStep::EnableAccount => script_steps::accounts::sanitize_enable_account(step),
        ScriptStep::ReLogin => script_steps::accounts::sanitize_re_login(step, obfuscate),
        ScriptStep::ResetAccountPassword => {
            script_steps::accounts::sanitize_reset_account_password(step, obfuscate)
        }
        _ => script_steps::sanitize::from_node(step_id, step),
    };

    let step = step_sanitized.or_else(|| {
        eprintln!("Could not parse: {step_xml}");
        None
    })?;

    if is_enabled {
        Some(step)
    } else {
        Some(format!("// {step}"))
    }
}
