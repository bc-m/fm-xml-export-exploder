use quick_xml::events::Event;
use quick_xml::Reader;

use crate::script_steps::parameters::calculation::Calculation;
use crate::utils::attributes::get_attribute;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut calculation = String::new();
    let mut only_current_file = false;

    let mut reader = Reader::from_str(step);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => name = get_attribute(&e, "name").unwrap().to_string(),
                b"Name" => {
                    match get_attribute(&e, "current").unwrap().as_str() {
                        "True" => {
                            only_current_file = true;
                        }
                        "False" => {
                            only_current_file = false;
                        }
                        _ => {}
                    };
                }
                b"Calculation" => calculation = Calculation::from_xml(&mut reader, &e).unwrap(),
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if name.is_empty() {
        None
    } else if calculation.is_empty() {
        Some(name.to_string())
    } else if only_current_file {
        Some(format!("{} [ Name: {} ; Current file ]", name, calculation))
    } else {
        Some(format!("{} [ Name: {} ]", name, calculation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_close_current() {
        let xml = r#"
            <Step index="12" id="121" name="Fenster schließen" enable="True">
                <UUID>A85B35DE-CF80-41B0-8E2D-F01DEF157FFF</UUID>
                <SourceUUID>B44BF438-B30C-4E90-A410-119377690950</SourceUUID>
                <OwnerID></OwnerID>
                <Options>0</Options>
                <ParameterValues membercount="1">
                    <Parameter type="WindowReference">
                        <WindowReference>
                            <Select kind="0" type="current"></Select>
                        </WindowReference>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Fenster schließen".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_close_by_name() {
        let xml = r#"
            <Step index="13" id="121" name="Fenster schließen" enable="True">
                <UUID>0A13A686-0AEF-4A1F-954E-AA68DBD0B028</UUID>
                <OwnerID></OwnerID>
                <Options>16384</Options>
                <ParameterValues membercount="1">
                    <Parameter type="WindowReference">
                        <WindowReference>
                            <Select kind="1" type="Calculated">
                                <Name current="False">
                                    <Calculation datatype="1" position="0">
                                        <Calculation>
                                            <Text><![CDATA["Foo Bar"]]></Text>
                                            <ChunkList hash="525D18B1E8FB2D7DFFF28F99FBDA6054">
                                                <Chunk type="NoRef">&quot;Foo Bar&quot;</Chunk>
                                            </ChunkList>
                                        </Calculation>
                                    </Calculation>
                                </Name>
                            </Select>
                        </WindowReference>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(r#"Fenster schließen [ Name: "Foo Bar" ]"#.to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_close_only_current_file() {
        let xml = r#"
            <Step index="13" id="121" name="Fenster schließen" enable="True">
                <UUID>0A13A686-0AEF-4A1F-954E-AA68DBD0B028</UUID>
                <OwnerID></OwnerID>
                <Options>16384</Options>
                <ParameterValues membercount="1">
                    <Parameter type="WindowReference">
                        <WindowReference>
                            <Select kind="1" type="Calculated">
                                <Name current="True">
                                    <Calculation datatype="1" position="0">
                                        <Calculation>
                                            <Text><![CDATA["Foo Bar"]]></Text>
                                            <ChunkList hash="525D18B1E8FB2D7DFFF28F99FBDA6054">
                                                <Chunk type="NoRef">&quot;Foo Bar&quot;</Chunk>
                                            </ChunkList>
                                        </Calculation>
                                    </Calculation>
                                </Name>
                            </Select>
                        </WindowReference>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some(r#"Fenster schließen [ Name: "Foo Bar" ; Current file ]"#.to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
