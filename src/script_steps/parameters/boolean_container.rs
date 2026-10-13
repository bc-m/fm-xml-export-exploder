use crate::script_steps::parameters::boolean::Boolean;
use crate::utils::xml_node::Node;

/// Window options stored as text, e.g. `<Resize>False</Resize>` becomes `Resize: OFF`
pub fn display(node: &Node) -> Option<String> {
    // Hide DimParentWindow entirely
    if node.name == "DimParentWindow" {
        return None;
    }

    let value = match node.text() {
        "True" => true,
        "False" => false,
        _ => return None,
    };

    // Hide other window options when they are ON (their default)
    if value
        && matches!(
            node.name.as_str(),
            "Close" | "Minimize" | "Maximize" | "Resize" | "MenuBar" | "Toolbar"
        )
    {
        return None;
    }

    let label = node.name.replace("MenuBar", "Menu");
    Some(format!("{label}: {}", Boolean::on_off(value)))
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::boolean_container;

    #[test]
    fn test() {
        let xml = r#"<Close>False</Close>"#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "Close: OFF".to_string();
        assert_eq!(
            boolean_container::display(&element).unwrap_or_default(),
            expected_output
        );
    }
}
