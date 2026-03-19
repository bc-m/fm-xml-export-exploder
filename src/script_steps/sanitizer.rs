use crate::config::Flags;
use crate::script_steps;
use crate::script_steps::constants::{ScriptStep, id_to_script_step};

pub fn sanitize(step_id: u32, step_xml: &str, flags: &Flags) -> Option<String> {
    let is_enabled = script_steps::is_enabled::sanitize(step_xml);
    let obfuscate = flags.obfuscate_passwords;

    let step_sanitized = match id_to_script_step(step_id) {
        ScriptStep::PerformScript => script_steps::perform_script::sanitize(step_xml),
        ScriptStep::GoToRecordRequestPage => script_steps::go_to_record::sanitize(step_xml),
        ScriptStep::OmitMultipleRecords => script_steps::omit_multiple_records::sanitize(step_xml),
        ScriptStep::PerformFind | ScriptStep::ConstrainFoundSet | ScriptStep::ExtendFoundSet => {
            script_steps::perform_find::sanitize(step_xml)
        }
        ScriptStep::InsertText => script_steps::insert_text::sanitize(step_xml),
        ScriptStep::SetField => script_steps::set_field_data::sanitize(step_xml),
        ScriptStep::ReplaceFieldContents => {
            script_steps::replace_field_contents::sanitize(step_xml)
        }
        ScriptStep::GoToPortalRow => script_steps::go_to_portal_row::sanitize(step_xml),
        ScriptStep::ExitScript => script_steps::exit_script::sanitize(step_xml),
        ScriptStep::CloseWindow => script_steps::close_window::sanitize(step_xml),
        ScriptStep::SetVariable => script_steps::set_variable::sanitize(step_xml),
        ScriptStep::GoToObject => script_steps::go_to_object::sanitize(step_xml),
        ScriptStep::RefreshObject => script_steps::refresh_object::sanitize(step_xml),
        ScriptStep::AddAccount => script_steps::accounts::sanitize_add_account(step_xml, obfuscate),
        ScriptStep::ChangePassword => {
            script_steps::accounts::sanitize_change_password(step_xml, obfuscate)
        }
        ScriptStep::DeleteAccount => script_steps::accounts::sanitize_delete_account(step_xml),
        ScriptStep::EnableAccount => script_steps::accounts::sanitize_enable_account(step_xml),
        ScriptStep::ReLogin => script_steps::accounts::sanitize_re_login(step_xml, obfuscate),
        ScriptStep::ResetAccountPassword => {
            script_steps::accounts::sanitize_reset_account_password(step_xml, obfuscate)
        }
        _ => script_steps::sanitize::from_xml(step_id, step_xml),
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
