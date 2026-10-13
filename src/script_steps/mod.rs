pub(crate) mod constants;
pub(crate) mod sanitizer;

mod accounts;
mod close_window;
mod exit_script;
mod go_to_object;
mod go_to_portal_row;
mod go_to_record;
mod insert_from_url;
mod insert_text;
mod parameters;
mod perform_script;
mod refresh_object;
mod replace_field_contents;
mod sanitize;
mod set_field_data;
mod set_variable;

/// How a step without parameters is shown
#[derive(Clone, Copy)]
pub(crate) enum WithoutParameters {
    /// e.g. `Commit Records/Requests`
    NameOnly,
    /// e.g. `Exit Script []`
    EmptyBrackets,
}

/// Formats a script step line, e.g. `Go to Layout [ Layout: "Start" ; Animation: None ]`
pub(crate) fn format_step(name: &str, parts: &[String], without: WithoutParameters) -> String {
    match (parts.is_empty(), without) {
        (false, _) => format!("{name} [ {} ]", parts.join(" ; ")),
        (true, WithoutParameters::NameOnly) => name.to_string(),
        (true, WithoutParameters::EmptyBrackets) => format!("{name} []"),
    }
}
