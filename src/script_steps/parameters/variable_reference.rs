use crate::utils::xml_node::Node;

/// e.g. `$var` or `$var[2]`
pub fn display(variable: &Node) -> Option<String> {
    let name = variable.attr("value")?;
    let repetition = variable
        .descendants()
        .into_iter()
        .skip(1)
        .filter(|n| n.name == "repetition")
        .filter_map(|n| n.attr("value")?.parse::<i32>().ok())
        .next_back();
    Some(match repetition {
        Some(rep) if rep != 1 => format!("{name}[{rep}]"),
        _ => name.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::variable_reference;

    #[test]
    fn test() {
        let xml = r#"
            <Variable value="$foo">
                <repetition value="1"></repetition>
            </Variable>
        "#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "$foo".to_string();
        assert_eq!(
            variable_reference::display(&element).unwrap(),
            expected_output
        );
    }

    #[test]
    fn test_with_repetition() {
        let xml = r#"
            <Variable value="$foo">
                <repetition value="1337"></repetition>
            </Variable>
        "#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "$foo[1337]".to_string();
        assert_eq!(
            variable_reference::display(&element).unwrap(),
            expected_output
        );
    }
}
