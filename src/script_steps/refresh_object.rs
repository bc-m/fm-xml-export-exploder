use crate::utils::attributes::get_attribute;
use crate::utils::xml_utils;
use quick_xml::events::Event;
use quick_xml::Reader;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut in_object_name_calculation = false;
    let mut in_repetition_calculation = false;
    let mut object_name_calculation = String::new();
    let mut repetition_calculation = String::new();

    let mut reader = Reader::from_str(step);
    reader.trim_text(true);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => name = get_attribute(&e, "name").unwrap().to_string(),
                b"Name" => in_object_name_calculation = true,
                b"repetition" => in_repetition_calculation = true,
                _ => {}
            },
            Ok(Event::CData(e)) => {
                if in_object_name_calculation {
                    object_name_calculation.push_str(xml_utils::cdata_to_string(&e).as_str());
                }
                if in_repetition_calculation {
                    repetition_calculation.push_str(xml_utils::cdata_to_string(&e).as_str());
                }
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"Calculation" {
                    in_object_name_calculation = false;
                    in_repetition_calculation = false;
                }
            }
            _ => {}
        }
        buf.clear()
    }

    let mut params = Vec::new();
    if !object_name_calculation.is_empty() && object_name_calculation != "1" {
        params.push(format!("Name: {}", object_name_calculation).clone());
    }
    if !repetition_calculation.is_empty() && repetition_calculation != "1" {
        params.push(repetition_calculation);
    }

    let params = params.join(" ; ");
    if params.is_empty() {
        Some(format!("{} []", name))
    } else {
        Some(format!("{} [ {} ]", name, params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize() {
        let xml_input = "
        <Step index=\"1\" id=\"167\" name=\"Objekt aktualisieren\" enable=\"True\">
            <UUID>B15C461D-629E-4EE6-990A-81AC5D203DA2</UUID>
            <OwnerID></OwnerID>
            <Options>0</Options>
            <ParameterValues membercount=\"1\">
                <Parameter type=\"Object\">
                    <Name></Name>
                    <repetition></repetition>
                </Parameter>
            </ParameterValues>
        </Step>
        ";

        let expected_output = Some("Objekt aktualisieren []".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_sanitize_by_name() {
        let xml_input = "
        <Step index=\"2\" id=\"167\" name=\"Objekt aktualisieren\" enable=\"True\">
            <UUID>8A55155F-0DEF-40C6-842B-5A8DDE9E2289</UUID>
            <OwnerID></OwnerID>
            <Options>16384</Options>
            <ParameterValues membercount=\"1\">
                <Parameter type=\"Object\">
                    <Name>
                        <Calculation datatype=\"1\" position=\"0\">
                            <Calculation>
                                <Text><![CDATA[\"Foo\"]]></Text>
                                <ChunkList hash=\"49C67A24DD1D8846365084DF7F1C0809\">
                                    <Chunk type=\"NoRef\">&quot;Foo&quot;</Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </Name>
                    <repetition>
                        <Calculation datatype=\"1\" position=\"1\">
                            <Calculation>
                                <Text><![CDATA[1]]></Text>
                                <ChunkList hash=\"A93F17BC54CA1958073B692697D5ED21\">
                                    <Chunk type=\"NoRef\">1</Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </repetition>
                </Parameter>
            </ParameterValues>
        </Step>
        ";

        let expected_output = Some("Objekt aktualisieren [ Name: \"Foo\" ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_sanitize_by_name_with_rep() {
        let xml_input = "
        <Step index=\"3\" id=\"167\" name=\"Objekt aktualisieren\" enable=\"True\">
            <UUID>6E838F90-D32C-480C-B0C9-3D6510F0904F</UUID>
            <OwnerID></OwnerID>
            <Options>16384</Options>
            <ParameterValues membercount=\"1\">
                <Parameter type=\"Object\">
                    <Name>
                        <Calculation datatype=\"1\" position=\"0\">
                            <Calculation>
                                <Text><![CDATA[\"Foo\"]]></Text>
                                <ChunkList hash=\"49C67A24DD1D8846365084DF7F1C0809\">
                                    <Chunk type=\"NoRef\">&quot;Foo&quot;</Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </Name>
                    <repetition>
                        <Calculation datatype=\"1\" position=\"1\">
                            <Calculation>
                                <Text><![CDATA[2]]></Text>
                                <ChunkList hash=\"34108238C823509F293F7FC877DB5DF6\">
                                    <Chunk type=\"NoRef\">2</Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </repetition>
                </Parameter>
            </ParameterValues>
        </Step>
        ";

        let expected_output = Some("Objekt aktualisieren [ Name: \"Foo\" ; 2 ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }
}
