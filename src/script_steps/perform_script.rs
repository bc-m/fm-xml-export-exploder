use quick_xml::events::Event;
use quick_xml::Reader;

use crate::script_steps::parameters::calculation::Calculation;
use crate::utils::attributes::get_attribute;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut script_reference_type = String::new();
    let mut script_reference_type_id = String::new();
    let mut script_reference = String::new();
    let mut calculation = String::new();

    let mut reader = Reader::from_str(step);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => {
                    name = get_attribute(&e, "name").unwrap().to_string();
                }
                b"List" => {
                    script_reference_type_id = get_attribute(&e, "value").unwrap().to_string();
                    script_reference_type = get_attribute(&e, "name").unwrap().to_string();
                    if script_reference_type_id.as_str() == "2" {
                        script_reference = Calculation::from_xml(&mut reader, &e)
                            .unwrap()
                            .display()
                            .unwrap();
                    }
                }
                b"ScriptReference" => {
                    script_reference = get_attribute(&e, "name").unwrap().to_string();
                }
                b"Parameter" => {
                    if get_attribute(&e, "type").unwrap_or("".to_string()).as_str() == "Parameter" {
                        calculation = Calculation::from_xml(&mut reader, &e)
                            .unwrap()
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

    if script_reference_type_id.as_str() != "2" {
        script_reference = format!("\"{}\"", script_reference);
    }

    if name.is_empty() {
        println!("empty primitive");
        None
    } else if calculation.is_empty() {
        Some(format!(
            "{} [ {} ; {} ]",
            name, script_reference_type, script_reference
        ))
    } else {
        Some(format!(
            "{} [ {} ; {} ; Parameter: {} ]",
            name, script_reference_type, script_reference, calculation
        ))
    }
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

        let expected_output =
            Some(r#"Script ausführen [ Aus Liste ; "Do something" ]"#.to_string());
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

        let expected_output = Some(r#"Script ausführen [ Aus Liste ; "Do something" ; Parameter: cf_ScriptparameterSetzen ( "CurlId" ; $curl ) ]"#.to_string());
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

        let expected_output =
            Some(r#"Script ausführen [ Nach Name ; "Do something" ; Parameter: 123 ]"#.to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
