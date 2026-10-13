use crate::script_steps::parameters::field_reference::FieldReference;
use crate::script_steps::parameters::variable_reference;
use crate::utils::xml_node::Node;

/// The target variable or field, e.g. `$result` or `Table::Field`
pub fn display(parameter: &Node) -> Option<String> {
    let mut target = None;
    visit(parameter, &mut target);
    target
}

fn visit(node: &Node, target: &mut Option<String>) {
    for child in &node.children {
        match child.name.as_str() {
            "Variable" => *target = variable_reference::display(child),
            "FieldReference" => *target = Some(FieldReference::from_node(child).display()),
            _ => visit(child, target),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::target;

    #[test]
    fn test() {
        let xml = r#"
            <Parameter type="Target">
                <Variable value="$foo">
                    <repetition value="1"></repetition>
                </Variable>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "$foo".to_string();
        assert_eq!(target::display(&element).unwrap(), expected_output);
    }
}
