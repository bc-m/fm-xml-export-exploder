use crate::utils::xml_node::Node;

/// Show Custom Dialog buttons, e.g. `Default Button: "OK"`
pub fn display(parameter: &Node, button_type: &str) -> Option<String> {
    // The label is either a plain attribute (shown quoted) or a calculation (shown as is)
    let (label, from_calculation) = match parameter.attr("value") {
        Some(label) => (label.to_string(), false),
        None => (
            parameter
                .text_cdata()
                .into_iter()
                .find(|text| !text.is_empty())?
                .to_string(),
            true,
        ),
    };
    if label.is_empty() {
        return None;
    }

    let name = match button_type {
        "Button1" => "Default Button",
        "Button2" => "Button 2",
        "Button3" => "Button 3",
        _ => button_type,
    };

    if from_calculation {
        Some(format!("{name}: {label}"))
    } else {
        Some(format!("{name}: \"{label}\""))
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::button;

    #[test]
    fn test_button_with_calculation() {
        let xml = r#"
            <Parameter type="Button1">
                <Calculation datatype="1" position="5">
                    <Calculation>
                        <Text><![CDATA["OK"]]></Text>
                    </Calculation>
                </Calculation>
                <Boolean type="Commit" value="False"></Boolean>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();
        assert_eq!(
            button::display(&element, "Button1"),
            Some(r#"Default Button: "OK""#.to_string())
        );
    }

    #[test]
    fn test_button_with_value_attribute() {
        let xml = r#"
            <Parameter type="Button1" value="Save">
                <Boolean type="Commit" value="True"></Boolean>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();
        assert_eq!(
            button::display(&element, "Button2"),
            Some(r#"Button 2: "Save""#.to_string())
        );
    }

    #[test]
    fn test_empty_button() {
        let xml = r#"
            <Parameter type="Button2">
                <Boolean type="Commit" value="False"></Boolean>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();
        assert_eq!(button::display(&element, "Button2"), None);
    }

    #[test]
    fn test_button3_display() {
        let xml = r#"
            <Parameter type="Button3" value="Maybe">
                <Boolean type="Commit" value="False"></Boolean>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();
        assert_eq!(
            button::display(&element, "Button3"),
            Some(r#"Button 3: "Maybe""#.to_string())
        );
    }
}
