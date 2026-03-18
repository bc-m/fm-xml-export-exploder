use quick_xml::Reader;
use quick_xml::events::Event;

use crate::script_steps::parameters::calculation::Calculation;
use crate::utils::attributes::{get_attribute, parse_unescaped_attribute};

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut data_source_reference: Option<String> = None;
    let mut script_reference_type = String::new();
    let mut script_reference_type_id = String::new();
    let mut script_reference = String::new();
    let mut calculation = String::new();

    let mut reader = Reader::from_str(step);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => {
                    name = get_attribute(&e, "name").unwrap();
                }
                b"List" => {
                    script_reference_type_id = get_attribute(&e, "value").unwrap();
                    script_reference_type = get_attribute(&e, "name").unwrap();
                    if script_reference_type_id == "2" {
                        script_reference =
                            Calculation::from_xml(&mut reader, &e).display().unwrap();
                    }
                }
                b"DataSourceReference" => {
                    data_source_reference = parse_unescaped_attribute(&e, "name")
                }
                b"ScriptReference" => script_reference = parse_unescaped_attribute(&e, "name")?,
                b"Parameter" => {
                    if get_attribute(&e, "type").as_deref() == Some("Parameter") {
                        calculation = Calculation::from_xml(&mut reader, &e)
                            .display()
                            .unwrap_or_default();
                    }
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if name.is_empty() {
        return None;
    }

    let mut parameters = Vec::new();

    if script_reference_type_id.as_str() == "2" {
        // By name: "Specified: By name" first, then the expression
        parameters.push(format!("Specified: {script_reference_type}"));
        parameters.push(script_reference.to_string());
    } else {
        // From list: quoted script name first, then "Specified: From list"
        parameters.push(format!("\"{script_reference}\""));
        parameters.push(format!("Specified: {script_reference_type}"));
    }

    if let Some(ref data_source_value) = data_source_reference {
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
        assert_eq!(sanitize(xml.trim()), expected_output);
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
        assert_eq!(sanitize(xml.trim()), expected_output);
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
        assert_eq!(sanitize(xml.trim()), expected_output);
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
        assert_eq!(sanitize(xml.trim()), expected_output);
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
        assert_eq!(sanitize(xml.trim()), expected_output);
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
        assert_eq!(sanitize(xml.trim()), expected_output);
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
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
