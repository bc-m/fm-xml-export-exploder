use crate::utils::xml_node::Node;

pub fn display(parameter: &Node) -> Option<String> {
    parameter
        .last_descendant("PrivilegeSetReference")?
        .attr("name")
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::privilege_set_reference;

    #[test]
    fn test_known() {
        let xml = r#"
            <Parameter type="PrivilegeSetReference">
                <PrivilegeSetReference id="2" name="[Data Entry Only]"></PrivilegeSetReference>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "[Data Entry Only]".to_string();
        assert_eq!(
            privilege_set_reference::display(&element).unwrap(),
            expected_output
        );
    }

    #[test]
    fn test_unknown() {
        let xml = r#"
            <Parameter type="PrivilegeSetReference">
                <PrivilegeSetReference id="0" name="&lt;unknown&gt;"></PrivilegeSetReference>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "<unknown>".to_string();
        assert_eq!(
            privilege_set_reference::display(&element).unwrap(),
            expected_output
        );
    }
}
