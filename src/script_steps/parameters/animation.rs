use crate::utils::xml_node::Node;

/// e.g. `Animation: Slide in from Left`
pub fn display(parameter: &Node) -> Option<String> {
    let name = parameter.last_descendant("Animation")?.attr("name")?;
    Some(format!("Animation: {name}"))
}
