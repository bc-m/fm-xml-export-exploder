use crate::utils::xml_node::Node;

pub fn display(node: &Node) -> Option<String> {
    node.attr("value").map(str::to_string)
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::text;

    #[test]
    fn test() {
        let xml = r#"<Text value="a&#13;b&#13;c"></Text>"#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "a\rb\rc".to_string();
        assert_eq!(text::display(&element).unwrap(), expected_output);
    }
}
