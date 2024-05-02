use quick_xml::events::Event;
use quick_xml::Reader;

use crate::script_steps::parameters::calculation::Calculation;
use crate::utils::attributes::get_attribute;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();

    let mut option = String::new();
    let mut option_type = String::new();
    let mut boolean_option_type = String::new();
    let mut boolean_option_value = false;
    let mut calculation = String::new();

    let mut reader = Reader::from_str(step);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => name = get_attribute(&e, "name").unwrap().to_string(),
                b"List" => {
                    option = get_attribute(&e, "name").unwrap().to_string();
                    option_type = get_attribute(&e, "value").unwrap().to_string();
                }
                b"Boolean" => {
                    boolean_option_type = get_attribute(&e, "type").unwrap().to_string();
                    boolean_option_value = get_attribute(&e, "value").unwrap() == "True";
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
    } else if option_type == "5" {
        if boolean_option_type.is_empty() {
            Some(format!("{} [ {} ]", name, calculation))
        } else {
            Some(format!(
                "{} [ {}: {} ; {} ]",
                name,
                boolean_option_type,
                match boolean_option_value {
                    true => "ON".to_string(),
                    false => "OFF".to_string(),
                },
                calculation
            ))
        }
    } else if boolean_option_type.is_empty() {
        Some(format!("{} [ {} ]", name, option))
    } else {
        Some(format!(
            "{} [ {} ; {}: {} ]",
            name,
            option,
            boolean_option_type,
            match boolean_option_value {
                true => "ON".to_string(),
                false => "OFF".to_string(),
            }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_to_first() {
        let xml = r#"
            <Step index="148" id="16" name="Gehe zu Datens./Abfrage/Seite" enable="True">
                <UUID>585E2A34-2216-49D9-92F0-8FBFC5E4DEFC</UUID>
                <OwnerID></OwnerID>
                <Options>2</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Records">
                        <List name="Erste(r)" value="1"></List>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Gehe zu Datens./Abfrage/Seite [ Erste(r) ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_go_to_last() {
        let xml = r#"
            <Step index="149" id="16" name="Gehe zu Datens./Abfrage/Seite" enable="True">
                <UUID>FEDAA48D-C6B3-4BC0-A483-EBCDE779581D</UUID>
                <OwnerID></OwnerID>
                <Options>2</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Records">
                        <List name="Letzte(r)" value="2"></List>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Gehe zu Datens./Abfrage/Seite [ Letzte(r) ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_go_to_previous_and_stop() {
        let xml = r#"
            <Step index="150" id="16" name="Gehe zu Datens./Abfrage/Seite" enable="True">
                <UUID>5D343B16-3241-4FC1-BEAB-FF8038A356F3</UUID>
                <OwnerID></OwnerID>
                <Options>8194</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Records">
                        <List name="Vorherige(r)" value="3">
                            <Boolean type="Nach letztem beenden" position="184" value="True"></Boolean>
                        </List>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(
            "Gehe zu Datens./Abfrage/Seite [ Vorherige(r) ; Nach letztem beenden: ON ]".to_string(),
        );
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_go_to_previous_not_stop() {
        let xml = r#"
            <Step index="150" id="16" name="Gehe zu Datens./Abfrage/Seite" enable="True">
                <UUID>5D343B16-3241-4FC1-BEAB-FF8038A356F3</UUID>
                <OwnerID></OwnerID>
                <Options>8194</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Records">
                        <List name="Vorherige(r)" value="3">
                            <Boolean type="Nach letztem beenden" position="184" value="False"></Boolean>
                        </List>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(
            "Gehe zu Datens./Abfrage/Seite [ Vorherige(r) ; Nach letztem beenden: OFF ]"
                .to_string(),
        );
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_go_to_next_and_stop() {
        let xml = r#"
            <Step index="152" id="16" name="Gehe zu Datens./Abfrage/Seite" enable="True">
                <UUID>8A3099C8-C500-4C96-B4AA-98DBC7FFD417</UUID>
                <OwnerID></OwnerID>
                <Options>8194</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Records">
                        <List name="Nächste(r)" value="4">
                            <Boolean type="Nach letztem beenden" position="184" value="True"></Boolean>
                        </List>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(
            "Gehe zu Datens./Abfrage/Seite [ Nächste(r) ; Nach letztem beenden: ON ]".to_string(),
        );
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_go_to_next_not_stop() {
        let xml = r#"
            <Step index="152" id="16" name="Gehe zu Datens./Abfrage/Seite" enable="True">
                <UUID>8A3099C8-C500-4C96-B4AA-98DBC7FFD417</UUID>
                <OwnerID></OwnerID>
                <Options>8194</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Records">
                        <List name="Nächste(r)" value="4">
                            <Boolean type="Nach letztem beenden" position="184" value="False"></Boolean>
                        </List>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(
            "Gehe zu Datens./Abfrage/Seite [ Nächste(r) ; Nach letztem beenden: OFF ]".to_string(),
        );
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_go_to_calculated() {
        let xml = r#"
            <Step index="154" id="16" name="Gehe zu Datens./Abfrage/Seite" enable="True">
                <UUID>88D1386F-3C9A-447C-8E01-F7518DA11D4E</UUID>
                <OwnerID></OwnerID>
                <Options>16514</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Records">
                        <List name="Nach Formel…" value="5">
                            <Boolean type="Mit Dialog" position="156" value="False"></Boolean>
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA[Hole( LayoutNummer ) + 1]]></Text>
                                    <ChunkList hash="92D1CC21320F97E522578F76FF6F2651">
                                        <Chunk type="FunctionRef">Hole</Chunk>
                                        <Chunk type="NoRef">( </Chunk>
                                        <Chunk type="FunctionRef">LayoutNummer</Chunk>
                                        <Chunk type="NoRef"> ) + 1</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </List>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(
            "Gehe zu Datens./Abfrage/Seite [ Mit Dialog: OFF ; Hole( LayoutNummer ) + 1 ]"
                .to_string(),
        );
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_go_to_calculated_with_dialog() {
        let xml = r#"
            <Step index="156" id="16" name="Gehe zu Datens./Abfrage/Seite" enable="True">
                <UUID>28A20FDE-D101-4019-977B-22516A869427</UUID>
                <OwnerID></OwnerID>
                <Options>16386</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Records">
                        <List name="Nach Formel…" value="5">
                            <Boolean type="Mit Dialog" position="156" value="True"></Boolean>
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA[$Nr]]></Text>
                                    <ChunkList hash="53D93C4D3455D9307C587F78123B7ED1">
                                        <Chunk type="VariableReference">$Nr</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </List>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some("Gehe zu Datens./Abfrage/Seite [ Mit Dialog: ON ; $Nr ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
