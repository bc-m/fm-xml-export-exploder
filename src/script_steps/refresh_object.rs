use crate::script_steps::{WithoutParameters, format_step};
use crate::utils::xml_node::Node;

pub fn sanitize(step: &Node) -> Option<String> {
    let name = step.attr("name").unwrap_or_default();
    // All CDATA inside <Name> and <repetition>, concatenated
    let cdata_below = |element: &str| -> String {
        step.descendants()
            .into_iter()
            .filter(|node| node.name == element)
            .flat_map(|node| node.descendants())
            .flat_map(|node| node.cdata.iter().map(String::as_str))
            .collect()
    };
    let object_name_calculation = cdata_below("Name");
    let repetition_calculation = cdata_below("repetition");

    let mut parts = Vec::new();
    if !object_name_calculation.is_empty() && object_name_calculation != "1" {
        parts.push(format!("Name: {object_name_calculation}"));
    }
    if !repetition_calculation.is_empty() && repetition_calculation != "1" {
        parts.push(repetition_calculation);
    }

    Some(format_step(name, &parts, WithoutParameters::EmptyBrackets))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::xml_node::Node;

    #[test]
    fn test() {
        let xml = r#"
            <Step index="1" id="167" name="Objekt aktualisieren" enable="True">
                <UUID>B15C461D-629E-4EE6-990A-81AC5D203DA2</UUID>
                <OwnerID></OwnerID>
                <Options>0</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Object">
                        <Name></Name>
                        <repetition></repetition>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Objekt aktualisieren []".to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }

    #[test]
    fn test_by_name() {
        let xml = r#"
            <Step index="2" id="167" name="Objekt aktualisieren" enable="True">
                <UUID>8A55155F-0DEF-40C6-842B-5A8DDE9E2289</UUID>
                <OwnerID></OwnerID>
                <Options>16384</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Object">
                        <Name>
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA["Foo"]]></Text>
                                    <ChunkList hash="49C67A24DD1D8846365084DF7F1C0809">
                                        <Chunk type="NoRef">&quot;Foo&quot;</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </Name>
                        <repetition>
                            <Calculation datatype="1" position="1">
                                <Calculation>
                                    <Text><![CDATA[1]]></Text>
                                    <ChunkList hash="A93F17BC54CA1958073B692697D5ED21">
                                        <Chunk type="NoRef">1</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </repetition>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(r#"Objekt aktualisieren [ Name: "Foo" ]"#.to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }

    #[test]
    fn test_by_name_with_rep() {
        let xml = r#"
            <Step index="3" id="167" name="Objekt aktualisieren" enable="True">
                <UUID>6E838F90-D32C-480C-B0C9-3D6510F0904F</UUID>
                <OwnerID></OwnerID>
                <Options>16384</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Object">
                        <Name>
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA["Foo"]]></Text>
                                    <ChunkList hash="49C67A24DD1D8846365084DF7F1C0809">
                                        <Chunk type="NoRef">&quot;Foo&quot;</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </Name>
                        <repetition>
                            <Calculation datatype="1" position="1">
                                <Calculation>
                                    <Text><![CDATA[2]]></Text>
                                    <ChunkList hash="34108238C823509F293F7FC877DB5DF6">
                                        <Chunk type="NoRef">2</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </repetition>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(r#"Objekt aktualisieren [ Name: "Foo" ; 2 ]"#.to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }
}
