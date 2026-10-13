use crate::utils::xml_node::{Node, Visit};

pub fn sanitize(step: &Node) -> Option<String> {
    let name = step.attr("name").unwrap_or_default();
    let mut calculation = String::new();
    let mut repetition = String::new();
    step.walk(&mut |node| match node.name.as_str() {
        "Name" => {
            calculation = node.calculation().unwrap_or_default();
            Visit::Skip
        }
        "repetition" => {
            repetition = node.calculation().unwrap_or_default();
            Visit::Skip
        }
        _ => Visit::Descend,
    });

    if name.is_empty() {
        return None;
    }
    if !calculation.is_empty() && !repetition.is_empty() {
        Some(format!(
            "{name} [ {calculation} ; Repetition: {repetition} ]"
        ))
    } else if !calculation.is_empty() {
        Some(format!("{name} [ {calculation} ]"))
    } else if !repetition.is_empty() {
        Some(format!("{name} [ ; Repetition: {repetition} ]"))
    } else {
        Some(format!("{name} []"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::xml_node::Node;

    #[test]
    fn test() {
        let xml = r#"
            <Step id="145" name="Gehe zu Objekt" enable="True">
                <SourceUUID>CAEB554C-8BA5-4074-BCEF-EAF86C7744A3</SourceUUID>
                <Options>16384</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Object">
                        <Name>
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA["Foo Bar"]]></Text>
                                    <ChunkList hash="DE83BCE7DDEC586C5CBA59FCA8CB05CC">
                                        <Chunk type="NoRef">&quot;Foo Bar&quot;</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </Name>
                        <repetition></repetition>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(r#"Gehe zu Objekt [ "Foo Bar" ]"#.to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }

    #[test]
    fn test_repetition() {
        let xml = r#"
            <Step id="145" name="Gehe zu Objekt" enable="True">
                <SourceUUID>CAEB554C-8BA5-4074-BCEF-EAF86C7744A3</SourceUUID>
                <Options>16384</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Object">
                        <Name>
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA["Foo Bar"]]></Text>
                                    <ChunkList hash="DE83BCE7DDEC586C5CBA59FCA8CB05CC">
                                        <Chunk type="NoRef">&quot;Foo Bar&quot;</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </Name>
                        <repetition>
                            <Calculation datatype="1" position="1">
                                <Calculation>
                                    <Text><![CDATA[$Rep]]></Text>
                                    <ChunkList hash="5EFB6D6F00E58E6BB5E399274B87A065">
                                        <Chunk type="VariableReference">$Rep</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </repetition>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some(r#"Gehe zu Objekt [ "Foo Bar" ; Repetition: $Rep ]"#.to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }
}
