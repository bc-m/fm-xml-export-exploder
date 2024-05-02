use quick_xml::events::Event;
use quick_xml::Reader;

use crate::script_steps::parameters::calculation::Calculation;
use crate::utils::attributes::get_attribute;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut variable_name = String::new();
    let mut value = String::new();
    let mut repetition = String::new();

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
                b"Name" => {
                    variable_name = get_attribute(&e, "value").unwrap().to_string();
                }
                b"value" => {
                    value = Calculation::from_xml(&mut reader, &e).unwrap();
                }
                b"repetition" => {
                    repetition = Calculation::from_xml(&mut reader, &e).unwrap();
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if name.is_empty() {
        None
    } else if repetition.is_empty() {
        Some(format!("{} [ {} ; {} ]", name, variable_name, value))
    } else {
        Some(format!(
            "{} [ {}[{}] ; {} ]",
            name, variable_name, repetition, value
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(sanitize(xml.trim()), expected_output);
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
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
