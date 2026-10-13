use crate::utils::xml_node::Node;

pub fn display(parameter: &Node) -> String {
    parameter
        .last_descendant("Comment")
        .and_then(|comment| comment.attr("value"))
        .unwrap_or_default()
        .to_string()
}
