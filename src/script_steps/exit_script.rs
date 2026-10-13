use crate::script_steps::{WithoutParameters, format_step};
use crate::utils::xml_node::{Node, Visit};

pub fn sanitize(step: &Node) -> Option<String> {
    let name = step.attr("name").unwrap_or_default();
    let mut calculation = String::new();
    step.walk(&mut |node| match node.name.as_str() {
        "Calculation" => {
            calculation = node.calculation().unwrap_or_default();
            Visit::Skip
        }
        _ => Visit::Descend,
    });

    if name.is_empty() {
        return None;
    }
    let parts: Vec<String> = Some(calculation)
        .filter(|c| !c.is_empty())
        .into_iter()
        .collect();
    Some(format_step(name, &parts, WithoutParameters::EmptyBrackets))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::xml_node::Node;

    #[test]
    fn test() {
        let xml = r#"<Step id="103" name="Aktuelles Script verlassen" enable="True"></Step>"#;

        let expected_output = Some("Aktuelles Script verlassen []".to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }

    #[test]
    fn test_with_calculation() {
        let xml = r#"
            <Step id="103" name="Aktuelles Script verlassen" enable="True">
                <Options>16384</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Calculation">
                        <Calculation datatype="1" position="0">
                            <Calculation>
                                <Text><![CDATA[$Foo]]></Text>
                                <ChunkList hash="3FC07F05CB535D7FBE7DBB56621B41CB">
                                    <Chunk type="VariableReference">$Foo</Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Aktuelles Script verlassen [ $Foo ]".to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }
}
