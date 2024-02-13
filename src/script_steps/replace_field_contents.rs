use crate::calculations::calculation::Calculation;
use crate::calculations::field_reference::FieldReferenceParameter;
use crate::utils::attributes::get_attribute;
use quick_xml::events::Event;
use quick_xml::Reader;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();

    let mut params: Vec<(String, String)> = Vec::new();
    let mut calculation = String::new();
    let mut calculation_label = String::new();

    let mut auto_increment_initial_value = String::new();
    let mut auto_increment_interval_value = String::new();
    let mut list_name_to_input_dialog_label = false;

    let mut reader = Reader::from_str(step);
    reader.trim_text(true);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => name = get_attribute(&e, "name").unwrap().to_string(),
                b"Boolean" => {
                    let value = get_attribute(&e, "value").unwrap().as_str() == "True";
                    if !value {
                        continue;
                    };
                    let label = get_attribute(&e, "type").unwrap();
                    params.push((
                        label,
                        match value {
                            true => "ON".to_string(),
                            false => "OFF".to_string(),
                        },
                    ));
                }
                b"Parameter" => {
                    if get_attribute(&e, "type").unwrap().as_str() == "FieldReference" {
                        let field_reference = FieldReferenceParameter::from_xml(&mut reader, &e)
                            .unwrap()
                            .field_reference;
                        params.push(("".to_string(), field_reference.to_string()))
                    }
                }
                b"Calculation" => {
                    calculation = Calculation::from_xml(&mut reader, &e).unwrap();
                }
                b"List" => {
                    if list_name_to_input_dialog_label {
                        if get_attribute(&e, "value").unwrap() == "True" {
                            if let Some(last_param) = params.last_mut() {
                                last_param.0 = last_param.1.clone();
                                last_param.1 = get_attribute(&e, "name").unwrap().to_string();
                            }
                        }
                        continue;
                    }

                    match get_attribute(&e, "value").unwrap_or_default().as_str() {
                        "0" | "1" => {
                            params.push(("".to_string(), get_attribute(&e, "name").unwrap()))
                        }
                        "2" => {
                            params.push(("".to_string(), get_attribute(&e, "name").unwrap()));
                            list_name_to_input_dialog_label = true;
                        }
                        "3" => calculation_label = get_attribute(&e, "name").unwrap(),
                        _ => {}
                    }
                }
                b"Initial" => auto_increment_initial_value = get_attribute(&e, "value").unwrap(),
                b"increment" => auto_increment_interval_value = get_attribute(&e, "value").unwrap(),
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if !calculation.is_empty() {
        params.push((calculation_label, calculation))
    }

    if !auto_increment_initial_value.is_empty() {
        params.push(("Initial".to_string(), auto_increment_initial_value))
    }
    if !auto_increment_interval_value.is_empty() {
        params.push(("Interval".to_string(), auto_increment_interval_value))
    }

    if name.is_empty() {
        None
    } else {
        let formatted_params: Vec<String> = params
            .iter()
            .map(|(key, value)| {
                if key.is_empty() {
                    value.replace(": ", "").to_string()
                } else {
                    format!("{}: {}", key.replace(": ", ""), value)
                }
            })
            .collect();

        Some(format!("{} [ {} ]", name, formatted_params.join(" ; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broken_reference() {
        let xml_input = "
        <Step id=\"91\" name=\"Ersetze alle Feldwerte\" enable=\"True\">
            <Options>1</Options>
            <ParameterValues membercount=\"3\">
                <Parameter type=\"Boolean\">
                    <Boolean type=\"Mit Dialog\" id=\"128\" value=\"True\"></Boolean>
                </Parameter>
                <Parameter type=\"FieldReference\">
                    <FieldReference id=\"0\" name=\"\">
                        <repetition value=\"1\"></repetition>
                    </FieldReference>
                </Parameter>
                <Parameter type=\"replace\">
                    <List name=\"Aktueller Inhalt\" value=\"0\"></List>
                </Parameter>
            </ParameterValues>
        </Step>";

        let expected_output = Some("Ersetze alle Feldwerte [ Mit Dialog: ON ; 🚨🚨🚨 BROKEN REFERENCE 🚨🚨🚨 ; Aktueller Inhalt ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_no_reference_with_calculation() {
        let xml_input = "
        <Step id=\"91\" name=\"Ersetze alle Feldwerte\" enable=\"True\">
            <Options>16384</Options>
            <ParameterValues membercount=\"2\">
                <Parameter type=\"Boolean\">
                    <Boolean type=\"Mit Dialog\" id=\"128\" value=\"True\"></Boolean>
                </Parameter>
                <Parameter type=\"replace\">
                    <List name=\"Durch Berechnung ersetzen: \" value=\"3\">
                        <Calculation datatype=\"1\" position=\"0\">
                            <Calculation>
                                <Text><![CDATA[123456]]></Text>
                                <ChunkList hash=\"BA8D9958B4673D27E78D4DB3506BD4CB\">
                                    <Chunk type=\"NoRef\">123456</Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </List>
                </Parameter>
            </ParameterValues>
        </Step>";

        let expected_output = Some(
            "Ersetze alle Feldwerte [ Mit Dialog: ON ; Durch Berechnung ersetzen: 123456 ]"
                .to_string(),
        );
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_with_reference() {
        let xml_input = "
        <Step id=\"91\" name=\"Ersetze alle Feldwerte\" enable=\"True\">
            <Options>5</Options>
            <ParameterValues membercount=\"3\">
                <Parameter type=\"Boolean\">
                    <Boolean type=\"Mit Dialog\" id=\"128\" value=\"True\"></Boolean>
                </Parameter>
                <Parameter type=\"FieldReference\">
                    <FieldReference id=\"6\" name=\"On\">
                        <repetition value=\"1\"></repetition>
                        <TableOccurrenceReference id=\"1065113\" name=\"_Syntax\"></TableOccurrenceReference>
                    </FieldReference>
                </Parameter>
                <Parameter type=\"replace\">
                    <List name=\"Aktueller Inhalt\" value=\"1\"></List>
                </Parameter>
            </ParameterValues>
        </Step>";

        let expected_output = Some(
            "Ersetze alle Feldwerte [ Mit Dialog: ON ; _Syntax::On ; Aktueller Inhalt ]"
                .to_string(),
        );
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_auto_increment_dialog() {
        let xml_input = "
        <Step id=\"91\" name=\"Ersetze alle Feldwerte\" enable=\"True\">
            <Options>5</Options>
            <ParameterValues membercount=\"3\">
                <Parameter type=\"Boolean\">
                    <Boolean type=\"Mit Dialog\" id=\"128\" value=\"True\"></Boolean>
                </Parameter>
                <Parameter type=\"FieldReference\">
                    <FieldReference id=\"6\" name=\"On\">
                        <repetition value=\"1\"></repetition>
                        <TableOccurrenceReference id=\"1065113\" name=\"_Syntax\"></TableOccurrenceReference>
                    </FieldReference>
                </Parameter>
                <Parameter type=\"replace\">
                    <List name=\"Durch fortlaufende Nummern ersetzen: \" value=\"2\">
                        <List name=\"Werte der Eingabeoptionen\" value=\"True\"></List>
                        <Boolean type=\"Eingabeoptionen aktualisieren\" value=\"False\"></Boolean>
                    </List>
                </Parameter>
            </ParameterValues>
        </Step>";

        let expected_output = Some("Ersetze alle Feldwerte [ Mit Dialog: ON ; _Syntax::On ; Durch fortlaufende Nummern ersetzen: Werte der Eingabeoptionen ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_auto_increment_fixed_values() {
        let xml_input = "
        <Step id=\"91\" name=\"Ersetze alle Feldwerte\" enable=\"True\">
            <Options>5</Options>
            <ParameterValues membercount=\"3\">
                <Parameter type=\"Boolean\">
                    <Boolean type=\"Mit Dialog\" id=\"128\" value=\"True\"></Boolean>
                </Parameter>
                <Parameter type=\"FieldReference\">
                    <FieldReference id=\"6\" name=\"On\">
                        <repetition value=\"1\"></repetition>
                        <TableOccurrenceReference id=\"1065113\" name=\"_Syntax\"></TableOccurrenceReference>
                    </FieldReference>
                </Parameter>
                <Parameter type=\"replace\">
                    <List name=\"Durch fortlaufende Nummern ersetzen: \" value=\"2\">
                        <List name=\"Werte der Eingabeoptionen\" value=\"False\">
                            <Initial value=\"1\"></Initial>
                            <increment value=\"1\"></increment>
                        </List>
                        <Boolean type=\"Eingabeoptionen aktualisieren\" value=\"False\"></Boolean>
                    </List>
                </Parameter>
            </ParameterValues>
        </Step>";

        let expected_output = Some("Ersetze alle Feldwerte [ Mit Dialog: ON ; _Syntax::On ; Durch fortlaufende Nummern ersetzen ; Initial: 1 ; Interval: 1 ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_auto_increment_and_update_options() {
        let xml_input = "
        <Step id=\"91\" name=\"Ersetze alle Feldwerte\" enable=\"True\">
            <Options>5</Options>
            <ParameterValues membercount=\"3\">
                <Parameter type=\"Boolean\">
                    <Boolean type=\"Mit Dialog\" id=\"128\" value=\"True\"></Boolean>
                </Parameter>
                <Parameter type=\"FieldReference\">
                    <FieldReference id=\"12\" name=\"__ID\">
                        <repetition value=\"1\"></repetition>
                        <TableOccurrenceReference id=\"1065113\" name=\"_Syntax\"></TableOccurrenceReference>
                    </FieldReference>
                </Parameter>
                <Parameter type=\"replace\">
                    <List name=\"Durch fortlaufende Nummern ersetzen: \" value=\"2\">
                        <List name=\"Werte der Eingabeoptionen\" value=\"False\">
                            <Initial value=\"100\"></Initial>
                            <increment value=\"10\"></increment>
                        </List>
                        <Boolean type=\"Eingabeoptionen aktualisieren\" value=\"True\"></Boolean>
                    </List>
                </Parameter>
            </ParameterValues>
        </Step>";

        let expected_output = Some("Ersetze alle Feldwerte [ Mit Dialog: ON ; _Syntax::__ID ; Durch fortlaufende Nummern ersetzen ; Eingabeoptionen aktualisieren: ON ; Initial: 100 ; Interval: 10 ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_auto_increment_dialog_given_start_and_interval() {
        let xml_input = "
        <Step id=\"91\" name=\"Ersetze alle Feldwerte\" enable=\"True\">
            <Options>5</Options>
            <ParameterValues membercount=\"3\">
                <Parameter type=\"Boolean\">
                    <Boolean type=\"Mit Dialog\" id=\"128\" value=\"True\"></Boolean>
                </Parameter>
                <Parameter type=\"FieldReference\">
                    <FieldReference id=\"12\" name=\"__ID\">
                        <repetition value=\"1\"></repetition>
                        <TableOccurrenceReference id=\"1065113\" name=\"_Syntax\"></TableOccurrenceReference>
                    </FieldReference>
                </Parameter>
                <Parameter type=\"replace\">
                    <List name=\"Durch fortlaufende Nummern ersetzen: \" value=\"2\">
                        <List name=\"Werte der Eingabeoptionen\" value=\"False\">
                            <Initial value=\"100\"></Initial>
                            <increment value=\"10\"></increment>
                        </List>
                        <Boolean type=\"Eingabeoptionen aktualisieren\" value=\"True\"></Boolean>
                    </List>
                </Parameter>
            </ParameterValues>
        </Step>";

        let expected_output = Some("Ersetze alle Feldwerte [ Mit Dialog: ON ; _Syntax::__ID ; Durch fortlaufende Nummern ersetzen ; Eingabeoptionen aktualisieren: ON ; Initial: 100 ; Interval: 10 ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_with_calculation() {
        let xml_input = "
        <Step id=\"91\" name=\"Ersetze alle Feldwerte\" enable=\"True\">
            <Options>16389</Options>
            <ParameterValues membercount=\"3\">
                <Parameter type=\"Boolean\">
                    <Boolean type=\"Mit Dialog\" id=\"128\" value=\"True\"></Boolean>
                </Parameter>
                <Parameter type=\"FieldReference\">
                    <FieldReference id=\"6\" name=\"On\">
                        <repetition value=\"1\"></repetition>
                        <TableOccurrenceReference id=\"1065113\" name=\"_Syntax\"></TableOccurrenceReference>
                    </FieldReference>
                </Parameter>
                <Parameter type=\"replace\">
                    <List name=\"Durch Berechnung ersetzen: \" value=\"3\">
                        <Calculation datatype=\"1\" position=\"0\">
                            <Calculation>
                                <Text><![CDATA[\"Calc\"]]></Text>
                                <ChunkList hash=\"B2984BE8225DC16063732A2220B18544\">
                                    <Chunk type=\"NoRef\">&quot;Calc&quot;</Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </List>
                </Parameter>
            </ParameterValues>
        </Step>";

        let expected_output =
            Some("Ersetze alle Feldwerte [ Mit Dialog: ON ; _Syntax::On ; Durch Berechnung ersetzen: \"Calc\" ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }
}
