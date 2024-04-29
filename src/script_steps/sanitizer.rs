use crate::script_steps;
use crate::script_steps::constants::{id_to_script_step, ScriptStep};

pub fn sanitize(step_id: &str, step_xml: &str) -> Option<String> {
    let is_enabled = script_steps::is_enabled::sanitize(step_xml);

    let step_sanitized = match id_to_script_step(step_id) {
        ScriptStep::PerformScript => script_steps::perform_script::sanitize(step_xml),
        ScriptStep::GoToPreviousField => script_steps::primitive::sanitize(step_xml),
        ScriptStep::GoToNextField => script_steps::primitive::sanitize(step_xml),
        ScriptStep::GoToLayout => script_steps::go_to_layout::sanitize(step_xml),
        ScriptStep::NewRecordRequest => script_steps::primitive::sanitize(step_xml),
        ScriptStep::DuplicateRecordRequest => script_steps::primitive::sanitize(step_xml),
        ScriptStep::DeleteRecordRequest => script_steps::primitive_with_boolean::sanitize(step_xml),
        ScriptStep::DeleteAllRecords => script_steps::primitive_with_boolean::sanitize(step_xml),
        ScriptStep::GoToRecordRequestPage => script_steps::go_to_record::sanitize(step_xml),
        ScriptStep::GoToField => script_steps::go_to_field::sanitize(step_xml),
        ScriptStep::CheckRecord => script_steps::primitive::sanitize(step_xml),
        ScriptStep::CheckFoundSet => script_steps::primitive::sanitize(step_xml),
        ScriptStep::UnsortRecords => script_steps::primitive::sanitize(step_xml),
        ScriptStep::EnterFindMode => script_steps::primitive_with_boolean::sanitize(step_xml),
        ScriptStep::ShowAllRecords => script_steps::primitive::sanitize(step_xml),
        ScriptStep::ModifyLastFind => script_steps::primitive::sanitize(step_xml),
        ScriptStep::OmitRecord => script_steps::primitive::sanitize(step_xml),
        ScriptStep::OmitMultipleRecords => script_steps::omit_multiple_records::sanitize(step_xml),
        ScriptStep::ShowOmittedOnly => script_steps::primitive::sanitize(step_xml),
        ScriptStep::PerformFind => script_steps::perform_find::sanitize(step_xml),
        ScriptStep::OpenHelp => script_steps::primitive::sanitize(step_xml),
        ScriptStep::OpenManageDatabase => script_steps::primitive::sanitize(step_xml),
        ScriptStep::ExitApplication => script_steps::primitive::sanitize(step_xml),
        ScriptStep::SelectAll => script_steps::primitive::sanitize(step_xml),
        ScriptStep::EnterBrowseMode => script_steps::primitive::sanitize(step_xml),
        ScriptStep::IfStart => script_steps::if_start::sanitize(step_xml),
        ScriptStep::Else => script_steps::primitive::sanitize(step_xml),
        ScriptStep::IfEnd => script_steps::primitive::sanitize(step_xml),
        ScriptStep::LoopStart => script_steps::primitive::sanitize(step_xml),
        ScriptStep::ExitLoopIf => script_steps::if_start::sanitize(step_xml),
        ScriptStep::LoopEnd => script_steps::primitive::sanitize(step_xml),
        ScriptStep::CommitRecordRequests => script_steps::commit::sanitize(step_xml),
        ScriptStep::SetField => script_steps::set_field_data::sanitize(step_xml),
        ScriptStep::FreezeWindow => script_steps::primitive::sanitize(step_xml),
        ScriptStep::NewFile => script_steps::primitive::sanitize(step_xml),
        ScriptStep::AllowUserAbort => script_steps::primitive_with_boolean::sanitize(step_xml),
        ScriptStep::SetErrorCapture => script_steps::primitive_with_boolean::sanitize(step_xml),
        ScriptStep::OpenScriptWorkspace => script_steps::primitive::sanitize(step_xml),
        ScriptStep::Comment => script_steps::comment::sanitize(step_xml),
        ScriptStep::HaltScript => script_steps::primitive::sanitize(step_xml),
        ScriptStep::ReplaceFieldContents => {
            script_steps::replace_field_contents::sanitize(step_xml)
        }
        ScriptStep::Beep => script_steps::primitive::sanitize(step_xml),
        ScriptStep::SetUseSystemFormats => script_steps::primitive_with_boolean::sanitize(step_xml),
        ScriptStep::GoToPortalRow => script_steps::go_to_portal_row::sanitize(step_xml),
        ScriptStep::CopyRecordRequest => script_steps::primitive::sanitize(step_xml),
        ScriptStep::FlushCacheToDisk => script_steps::primitive::sanitize(step_xml),
        ScriptStep::ExitScript => script_steps::exit_script::sanitize(step_xml),
        ScriptStep::OpenPreferences => script_steps::primitive::sanitize(step_xml),
        ScriptStep::CorrectWord => script_steps::primitive::sanitize(step_xml),
        ScriptStep::SpellingOptions => script_steps::primitive::sanitize(step_xml),
        ScriptStep::SelectDictionaries => script_steps::primitive::sanitize(step_xml),
        ScriptStep::OpenManageValueLists => script_steps::primitive::sanitize(step_xml),
        ScriptStep::OpenSharing => script_steps::primitive::sanitize(step_xml),
        ScriptStep::OpenFileOptions => script_steps::primitive::sanitize(step_xml),
        ScriptStep::AllowFormattingBar => script_steps::primitive_with_boolean::sanitize(step_xml),
        ScriptStep::OpenHosts => script_steps::primitive::sanitize(step_xml),
        ScriptStep::EditUserDictionary => script_steps::primitive::sanitize(step_xml),
        ScriptStep::CloseWindow => script_steps::close_window::sanitize(step_xml),
        ScriptStep::NewWindow => script_steps::new_window::sanitize(step_xml),
        ScriptStep::IfElse => script_steps::if_start::sanitize(step_xml),
        ScriptStep::ConstrainFoundSet => script_steps::perform_find::sanitize(step_xml),
        ScriptStep::ExtendFoundSet => script_steps::perform_find::sanitize(step_xml),
        ScriptStep::OpenFindReplace => script_steps::primitive::sanitize(step_xml),
        ScriptStep::OpenManageDataSources => script_steps::primitive::sanitize(step_xml),
        ScriptStep::SetVariable => script_steps::set_variable::sanitize(step_xml),
        ScriptStep::GoToObject => script_steps::go_to_object::sanitize(step_xml),
        ScriptStep::OpenEditSavedFinds => script_steps::primitive::sanitize(step_xml),
        ScriptStep::OpenManageLayouts => script_steps::primitive::sanitize(step_xml),
        ScriptStep::OpenManageContainers => script_steps::primitive::sanitize(step_xml),
        ScriptStep::OpenManageThemes => script_steps::primitive::sanitize(step_xml),
        ScriptStep::RefreshObject => script_steps::refresh_object::sanitize(step_xml),
        ScriptStep::ClosePopover => script_steps::primitive::sanitize(step_xml),
        ScriptStep::UploadToServer => script_steps::primitive::sanitize(step_xml),
        ScriptStep::OpenFavorites => script_steps::primitive::sanitize(step_xml),
        ScriptStep::Unknown => Option::from(format!(
            "{} ⚠️⚠️⚠️ FM-XML-EXPORT-EXPLODER: UNKNOWN SCRIPT-STEP [ ID: {:?} ] ⚠️⚠️⚠️",
            script_steps::primitive::sanitize(step_xml).unwrap(),
            step_id
        )),
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
