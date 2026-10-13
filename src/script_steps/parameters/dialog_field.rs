use crate::script_steps::parameters::target;
use crate::utils::xml_node::Node;

#[derive(Debug, Default)]
pub struct DialogField {
    pub target: Option<String>,
    pub label: Option<String>,
    pub password: bool,
}

impl DialogField {
    /// An input field of Show Custom Dialog (`<Parameter type="Field1">`)
    pub fn from_node(field: &Node) -> DialogField {
        let mut item = DialogField::default();
        for child in field.descendants().into_iter().skip(1) {
            match child.name.as_str() {
                "Parameter" => match child.attr("type") {
                    Some("Target") => item.target = target::display(child),
                    Some("Label") => item.label = child.calculation(),
                    _ => {}
                },
                // The only Boolean of an input field is the password flag. Its type
                // is localized ("Password", "Passwort", ...), so only the value counts.
                "Boolean" => item.password = child.attr("value") == Some("True"),
                _ => {}
            }
        }
        item
    }

    pub fn display(self, field_type: &str) -> Option<String> {
        let target = self.target?;
        let number = field_type.strip_prefix("Field").unwrap_or("?");

        let mut parts = vec![format!("Input {number}: {target}")];

        if let Some(label) = self.label
            && !label.is_empty()
        {
            parts.push(format!("Label {number}: {label}"));
        }

        if self.password {
            parts.push("Password".to_string());
        }

        Some(parts.join(" ; "))
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use super::DialogField;

    #[test]
    fn test_localized_password_flag() {
        let xml = r#"<Parameter type="Field3"><Parameter type="Target"><Variable value="$pw"><repetition value="1"></repetition></Variable></Parameter><Boolean type="Passwort" value="True"></Boolean><Parameter type="Label"><Calculation datatype="1" position="2"><Calculation><Text><![CDATA["PW"]]></Text></Calculation></Calculation></Parameter></Parameter>"#;

        let element = Node::parse(xml).unwrap();

        let field = DialogField::from_node(&element);
        assert_eq!(
            field.display("Field3"),
            Some(r#"Input 3: $pw ; Label 3: "PW" ; Password"#.to_string())
        );
    }

    #[test]
    fn test_field_with_variable_and_label() {
        let xml = r#"
            <Parameter type="Field1">
                <Parameter type="Target">
                    <Variable value="$input1">
                        <repetition>
                            <Calculation datatype="1" position="32">
                                <Calculation>
                                    <Text><![CDATA[1]]></Text>
                                </Calculation>
                            </Calculation>
                        </repetition>
                    </Variable>
                </Parameter>
                <Boolean type="Password" value="False"></Boolean>
                <Parameter type="Label">
                    <Calculation datatype="1" position="2">
                        <Calculation>
                            <Text><![CDATA["label1"]]></Text>
                        </Calculation>
                    </Calculation>
                </Parameter>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();

        let field = DialogField::from_node(&element);
        assert_eq!(field.target, Some("$input1".to_string()));
        assert_eq!(field.label, Some(r#""label1""#.to_string()));
        assert!(!field.password);
        assert_eq!(
            field.display("Field1"),
            Some(r#"Input 1: $input1 ; Label 1: "label1""#.to_string())
        );
    }

    #[test]
    fn test_field_with_password() {
        let xml = r#"
            <Parameter type="Field1">
                <Parameter type="Target">
                    <Variable value="$input1">
                        <repetition>
                            <Calculation datatype="1" position="32">
                                <Calculation>
                                    <Text><![CDATA[1]]></Text>
                                </Calculation>
                            </Calculation>
                        </repetition>
                    </Variable>
                </Parameter>
                <Boolean type="Password" value="True"></Boolean>
                <Parameter type="Label">
                    <Calculation datatype="1" position="2">
                        <Calculation>
                            <Text><![CDATA["label1"]]></Text>
                        </Calculation>
                    </Calculation>
                </Parameter>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();

        let field = DialogField::from_node(&element);
        assert!(field.password);
        assert_eq!(
            field.display("Field1"),
            Some(r#"Input 1: $input1 ; Label 1: "label1" ; Password"#.to_string())
        );
    }

    #[test]
    fn test_field2_display() {
        let xml = r#"
            <Parameter type="Field2">
                <Parameter type="Target">
                    <Variable value="$input2">
                        <repetition>
                            <Calculation datatype="1" position="33">
                                <Calculation>
                                    <Text><![CDATA[1]]></Text>
                                </Calculation>
                            </Calculation>
                        </repetition>
                    </Variable>
                </Parameter>
                <Boolean type="Password" value="False"></Boolean>
                <Parameter type="Label">
                    <Calculation datatype="1" position="3">
                        <Calculation>
                            <Text><![CDATA["second"]]></Text>
                        </Calculation>
                    </Calculation>
                </Parameter>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();

        let field = DialogField::from_node(&element);
        assert_eq!(
            field.display("Field2"),
            Some(r#"Input 2: $input2 ; Label 2: "second""#.to_string())
        );
    }
}
