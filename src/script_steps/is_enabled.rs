use crate::utils::xml_node::Node;

pub fn sanitize(step: &Node) -> bool {
    step.attr("enable") != Some("False")
}
