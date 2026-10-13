use crate::utils::xml_node::Node;

/// e.g. `Style: Document`
pub fn display(style: &Node) -> Option<String> {
    style.attr("name").map(|name| format!("Style: {name}"))
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::style;

    #[test]
    fn test() {
        let xml = r#"<Style name="Dokument" value="3221291010"></Style>"#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "Style: Dokument".to_string();
        assert_eq!(style::display(&element).unwrap(), expected_output);
    }
}
