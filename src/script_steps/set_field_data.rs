use crate::script_steps::parameters::field_reference::FieldReference;
use crate::utils::xml_node::{Node, Visit};

pub fn sanitize(step: &Node) -> Option<String> {
    let name = step.attr("name").unwrap_or_default();
    let mut field_reference = String::new();
    let mut calculation = String::new();
    step.walk(&mut |node| match node.name.as_str() {
        "FieldReference" => {
            field_reference = FieldReference::from_node(node).display();
            Visit::Skip
        }
        "Calculation" => {
            calculation = node.calculation().unwrap_or_default();
            Visit::Skip
        }
        _ => Visit::Descend,
    });

    if name.is_empty() {
        return None;
    }
    Some(format!("{name} [ {field_reference} ; {calculation} ]"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::xml_node::Node;

    #[test]
    fn test() {
        let xml = r#"
            <Step id="76" name="Feldwert setzen" enable="True">
                <Options>16385</Options>
                <ParameterValues membercount="2">
                    <Parameter type="FieldReference">
                        <FieldReference id="4" name="FieldFoo">
                            <repetition value="1"></repetition>
                            <TableOccurrenceReference id="1065090" name="TableFoo"></TableOccurrenceReference>
                        </FieldReference>
                    </Parameter>
                    <Parameter type="Calculation">
                        <Calculation datatype="1" position="0">
                            <Calculation>
                                <Text><![CDATA[TableBar::FieldBar]]></Text>
                                <ChunkList hash="83065BB32DCAC16FD7B0ABECE2D7A367">
                                    <Chunk type="FieldRef">
                                        <FieldReference id="4" name="FieldBar" repetition="1">
                                            <TableOccurrenceReference id="1065089" name="TableBar"></TableOccurrenceReference>
                                        </FieldReference>
                                    </Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some("Feldwert setzen [ TableFoo::FieldFoo ; TableBar::FieldBar ]".to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }
}
