use crate::utils::xml_node::{Node, Visit};

pub fn sanitize(step: &Node) -> Option<String> {
    let name = step.attr("name").unwrap_or_default();
    let mut data_source_reference: Option<&str> = None;
    let mut script_reference_type = "";
    let mut script_reference_type_id = "";
    let mut script_reference = String::new();
    let mut unnamed_script_reference = false;
    let mut calculation = String::new();

    step.walk(&mut |node| match node.name.as_str() {
        "List" => {
            script_reference_type_id = node.attr("value").unwrap_or_default();
            script_reference_type = node.attr("name").unwrap_or_default();
            if script_reference_type_id == "2" {
                // By name: the script name is a calculation
                script_reference = node.calculation().unwrap_or_default();
                return Visit::Skip;
            }
            Visit::Descend
        }
        "DataSourceReference" => {
            data_source_reference = node.attr("name");
            Visit::Descend
        }
        "ScriptReference" => {
            match node.attr("name") {
                Some(script_name) => script_reference = script_name.to_string(),
                // A script reference without a name can't be displayed
                None => unnamed_script_reference = true,
            }
            Visit::Descend
        }
        "Parameter" if node.attr("type") == Some("Parameter") => {
            calculation = node.calculation().unwrap_or_default();
            Visit::Skip
        }
        _ => Visit::Descend,
    });
    if unnamed_script_reference || name.is_empty() {
        return None;
    }

    let mut parameters = Vec::new();

    if script_reference_type_id == "2" {
        // By name: "Specified: By name" first, then the expression
        parameters.push(format!("Specified: {script_reference_type}"));
        parameters.push(script_reference);
    } else {
        // From list: quoted script name first, then "Specified: From list"
        parameters.push(format!("\"{script_reference}\""));
        parameters.push(format!("Specified: {script_reference_type}"));
    }

    if let Some(data_source_value) = data_source_reference {
        parameters.push(format!("File: \"{data_source_value}\""));
    }

    // Always include Parameter:, even when empty
    if calculation.is_empty() {
        parameters.push("Parameter:".to_string());
    } else {
        parameters.push(format!("Parameter: {calculation}"));
    }

    Some(format!("{name} [ {} ]", parameters.join(" ; ")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::xml_node::Node;

    #[test]
    fn test_without_parameter() {
        let xml = r#"
            <Step id="1" name="Script ausführen" enable="True">
                <Options>16448</Options>
                <ParameterValues membercount="2">
                    <Parameter type="List">
                        <List name="Aus Liste" value="1">
                            <ScriptReference id="101" name="Do something" UUID="5C78C418-8738-4F5F-88D2-224ED869EE57"></ScriptReference>
                        </List>
                    </Parameter>
                    <Parameter type="Parameter">
                        <Parameter></Parameter>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(
            r#"Script ausführen [ "Do something" ; Specified: Aus Liste ; Parameter: ]"#
                .to_string(),
        );
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }

    #[test]
    fn test_with_parameter() {
        let xml = r#"
            <Step id="1" name="Script ausführen" enable="True">
                <Options>16448</Options>
                <ParameterValues membercount="2">
                    <Parameter type="List">
                        <List name="Aus Liste" value="1">
                            <ScriptReference id="101" name="Do something" UUID="5C78C418-8738-4F5F-88D2-224ED869EE57"></ScriptReference>
                        </List>
                    </Parameter>
                    <Parameter type="Parameter">
                        <Parameter>
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA[cf_ScriptparameterSetzen ( "CurlId" ; $curl )]]></Text>
                                    <ChunkList hash="A0EB6D37EC72BB1046A486D5D41ABB9C">
                                        <Chunk type="CustomFunctionRef">cf_ScriptparameterSetzen</Chunk>
                                        <Chunk type="NoRef"> ( &quot;CurlId&quot; ; </Chunk>
                                        <Chunk type="VariableReference">$curl</Chunk>
                                        <Chunk type="NoRef"> )</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </Parameter>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(r#"Script ausführen [ "Do something" ; Specified: Aus Liste ; Parameter: cf_ScriptparameterSetzen ( "CurlId" ; $curl ) ]"#.to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }

    #[test]
    fn test_by_name_with_parameter() {
        let xml = r#"
            <Step index="8" id="1" name="Script ausführen" enable="True">
                <UUID>15EDD097-DB5C-4473-BC2A-16D6ECD50E75</UUID>
                <OwnerID></OwnerID>
                <Options>33572864</Options>
                <ParameterValues membercount="2">
                    <Parameter type="List">
                        <List name="Nach Name" value="2">
                            <Calculation datatype="1" position="2">
                                <Calculation>
                                    <Text><![CDATA["Do something"]]></Text>
                                    <ChunkList hash="24B36DBE9E0B1EA70AC43EDBB49221C3">
                                        <Chunk type="NoRef">&quot;Do something&quot;</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </List>
                    </Parameter>
                    <Parameter type="Parameter">
                        <Parameter>
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA[123]]></Text>
                                    <ChunkList hash="90213DC459B04C5A47C044B9460AEF7B">
                                        <Chunk type="NoRef">123</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </Parameter>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(
            r#"Script ausführen [ Specified: Nach Name ; "Do something" ; Parameter: 123 ]"#
                .to_string(),
        );
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }

    #[test]
    fn test_run_external_script() {
        let xml = r#"
            <Step id="1" name="Script ausführen" enable="True">
                <Options>80</Options>
                <ParameterValues membercount="2">
                    <Parameter type="List">
                        <List name="Aus Liste" value="1">
                            <DataSourceReference id="1" name="App_Utils"></DataSourceReference>
                            <ScriptReference id="724" name="Do something"></ScriptReference>
                        </List>
                    </Parameter>
                    <Parameter type="Parameter">
                        <Parameter>
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA[123]]></Text>
                                    <ChunkList hash="90213DC459B04C5A47C044B9460AEF7B">
                                        <Chunk type="NoRef">123</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </Parameter>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(
            r#"Script ausführen [ "Do something" ; Specified: Aus Liste ; File: "App_Utils" ; Parameter: 123 ]"#
                .to_string(),
        );
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }

    #[test]
    fn test_from_list_no_parameter() {
        let xml = r#"
            <Step hash="F103CFD9962DBB42A45BF539369561B8" index="0" id="1" name="Perform Script" enable="True">
                <Options>64</Options>
                <ParameterValues membercount="2">
                    <Parameter type="List">
                        <List name="From list" value="1">
                            <ScriptReference id="1" name="Perform Script" UUID="12F7E3B3-EDB6-43F7-BDB2-9A83B45CDA2B"></ScriptReference>
                        </List>
                    </Parameter>
                    <Parameter type="Parameter">
                        <Parameter></Parameter>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(
            r#"Perform Script [ "Perform Script" ; Specified: From list ; Parameter: ]"#
                .to_string(),
        );
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }

    #[test]
    fn test_from_list_with_parameter() {
        let xml = r#"
            <Step hash="FEB8AED7ED23D45418A2532C4FDC82BB" index="1" id="1" name="Perform Script" enable="True">
                <Options>16448</Options>
                <ParameterValues membercount="2">
                    <Parameter type="List">
                        <List name="From list" value="1">
                            <ScriptReference id="1" name="Perform Script" UUID="12F7E3B3-EDB6-43F7-BDB2-9A83B45CDA2B"></ScriptReference>
                        </List>
                    </Parameter>
                    <Parameter type="Parameter">
                        <Parameter>
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA["parameter"]]></Text>
                                </Calculation>
                            </Calculation>
                        </Parameter>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some(r#"Perform Script [ "Perform Script" ; Specified: From list ; Parameter: "parameter" ]"#.to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }

    #[test]
    fn test_by_name_with_parameter_english() {
        let xml = r#"
            <Step hash="89DD98FBBA64206AFA1A3B82F42F0F36" index="2" id="1" name="Perform Script" enable="True">
                <Options>33572928</Options>
                <ParameterValues membercount="2">
                    <Parameter type="List">
                        <List name="By name" value="2">
                            <Calculation datatype="1" position="2">
                                <Calculation>
                                    <Text><![CDATA["script name"]]></Text>
                                </Calculation>
                            </Calculation>
                        </List>
                    </Parameter>
                    <Parameter type="Parameter">
                        <Parameter>
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA["parameter"]]></Text>
                                </Calculation>
                            </Calculation>
                        </Parameter>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(
            r#"Perform Script [ Specified: By name ; "script name" ; Parameter: "parameter" ]"#
                .to_string(),
        );
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }
}
