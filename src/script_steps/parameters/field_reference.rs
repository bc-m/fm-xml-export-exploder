use crate::script_steps::parameters::format::{BROKEN_REFERENCE, BROKEN_REFERENCE_NAME};
use crate::utils::xml_node::Node;

#[derive(Debug, Default)]
pub struct FieldReference {
    pub table_reference: Option<String>,
    pub field_reference: Option<String>,
    pub repetition: Option<i32>,
}

impl FieldReference {
    /// Reads the field, its table occurrence and repetition at or below `node`;
    /// when there are several, the last one wins.
    pub fn from_node(node: &Node) -> FieldReference {
        let mut item = FieldReference {
            field_reference: node.attr("name").map(str::to_string),
            ..Default::default()
        };
        for child in node.descendants().into_iter().skip(1) {
            match child.name.as_str() {
                "FieldReference" => item.field_reference = child.attr("name").map(str::to_string),
                "TableOccurrenceReference" => {
                    item.table_reference = child.attr("name").map(str::to_string);
                }
                "repetition" => item.repetition = child.attr("value").and_then(|v| v.parse().ok()),
                _ => {}
            }
        }
        item
    }

    pub fn display(&self) -> String {
        let Some(table_reference) = &self.table_reference else {
            return BROKEN_REFERENCE.to_string();
        };

        let field_reference = match &self.field_reference {
            Some(reference) if !reference.is_empty() => reference.as_str(),
            _ => BROKEN_REFERENCE_NAME,
        };

        match self.repetition {
            Some(rep) if rep != 1 => format!("{table_reference}::{field_reference}[{rep}]"),
            _ => format!("{table_reference}::{field_reference}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::field_reference::FieldReference;

    #[test]
    fn test_field_reference() {
        let xml = r#"
            <FieldReference id="4" name="Bar">
                <repetition value="1"></repetition>
                <TableOccurrenceReference id="1065090" name="Foo"></TableOccurrenceReference>
            </FieldReference>
        "#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "Foo::Bar";
        assert_eq!(
            FieldReference::from_node(&element).display(),
            expected_output
        );
    }

    #[test]
    fn test_field_reference_with_repetition() {
        let xml = r#"
            <FieldReference id="4" name="Bar">
                <repetition value="1337"></repetition>
                <TableOccurrenceReference id="1065090" name="Foo"></TableOccurrenceReference>
            </FieldReference>
        "#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "Foo::Bar[1337]";
        assert_eq!(
            FieldReference::from_node(&element).display(),
            expected_output
        );
    }
}
