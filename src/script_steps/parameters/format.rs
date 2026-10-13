//! Small helpers shared by the parameter formatters.

use crate::script_steps::parameters::boolean::Boolean;
use crate::utils::xml_node::Node;

/// Shown instead of a field whose table occurrence is missing
pub(crate) const BROKEN_REFERENCE: &str = "🚨🚨🚨 BROKEN REFERENCE 🚨🚨🚨";
/// Shown instead of a missing name inside an otherwise valid reference
pub(crate) const BROKEN_REFERENCE_NAME: &str = "🚨🚨🚨 <BROKEN REFERENCE> 🚨🚨🚨";

pub(crate) fn on_off(node: &Node, attribute: &str) -> &'static str {
    Boolean::on_off(node.attr_is_true(attribute))
}

/// Text that FileMaker shows inside quotes, e.g. file paths.
pub(crate) fn quoted(text: &str) -> String {
    format!("\"{text}\"")
}

pub(crate) fn labeled(label: &str, value: Option<String>) -> Option<String> {
    value.map(|value| format!("{label}: {value}"))
}

/// Joins display parts with ` ; `; `None` when there is nothing to show.
pub(crate) fn join(parts: Vec<String>) -> Option<String> {
    Some(parts.join(" ; ")).filter(|display| !display.is_empty())
}
