use crate::utils::xml_node::{Node, Visit};

pub fn sanitize(step: &Node) -> Option<String> {
    let name = step.attr("name").unwrap_or_default();
    let mut variable_name = "";
    let mut value = String::new();
    let mut repetition = String::new();
    step.walk(&mut |node| match node.name.as_str() {
        "Name" => {
            variable_name = node.attr("value").unwrap_or_default();
            Visit::Descend
        }
        "value" => {
            value = node.calculation().unwrap_or_default();
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
    let var = if repetition.is_empty() {
        variable_name.to_string()
    } else {
        format!("{variable_name}[{repetition}]")
    };
    Some(format!("{name} [ {var} ; {value} ]"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::xml_node::Node;

    #[test]
    fn test() {
        let xml = r#"
            <Step id="141" name="Variable setzen" enable="True">
                <Options>16388</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Variable">
                        <value>
                            <Calculation datatype="1" position="1">
                                <Calculation>
                                    <Text><![CDATA["Bar"]]></Text>
                                    <ChunkList hash="421FDA48D19A43EC49954D2B558EDE55">
                                        <Chunk type="NoRef">&quot;Bar&quot;</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </value>
                        <Name value="$Foo"></Name>
                        <repetition></repetition>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(r#"Variable setzen [ $Foo ; "Bar" ]"#.to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }

    #[test]
    fn test_complex() {
        let xml = r#"
            <Step index="17" id="141" name="Variable setzen" enable="True">
                <UUID>E9ADFC26-91FA-4C6E-81A4-4979ECA90EF8</UUID>
                <OwnerID></OwnerID>
                <Options>16388</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Variable">
                        <value>
                            <Calculation datatype="1" position="1">
                                <Calculation>
                                    <Text><![CDATA[$Bar]]></Text>
                                    <ChunkList hash="44A9531ED48D23A847A9895B07F60D0C">
                                        <Chunk type="VariableReference">Bar</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </value>
                        <Name value="$Foo"></Name>
                        <repetition>
                            <Calculation datatype="1" position="2">
                                <Calculation>
                                    <Text><![CDATA[$Rep]]></Text>
                                    <ChunkList hash="E90AA30981F2F06DF29263D3DD430B97">
                                        <Chunk type="VariableReference">$Rep</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </repetition>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Variable setzen [ $Foo[$Rep] ; $Bar ]".to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }
}
