use crate::script_steps::parameters::boolean::Boolean;
use crate::utils::xml_node::{Node, Visit};

pub fn sanitize(step: &Node) -> Option<String> {
    let name = step.attr("name").unwrap_or_default();
    let mut option = "";
    let mut option_type = "";
    let mut boolean_option_type = "";
    let mut boolean_option_value = false;
    let mut calculation = String::new();
    step.walk(&mut |node| match node.name.as_str() {
        "List" => {
            option = node.attr("name").unwrap_or_default();
            option_type = node.attr("value").unwrap_or_default();
            Visit::Descend
        }
        "Boolean" => {
            boolean_option_type = node.attr("type").unwrap_or_default();
            boolean_option_value = node.attr_is_true("value");
            Visit::Descend
        }
        "Calculation" => {
            calculation = node.calculation().unwrap_or_default();
            Visit::Skip
        }
        _ => Visit::Descend,
    });

    if name.is_empty() {
        return None;
    }

    let is_calculated = option_type == "5";
    let boolean_option = (!boolean_option_type.is_empty()).then(|| {
        format!(
            "{boolean_option_type}: {}",
            Boolean::on_off(boolean_option_value)
        )
    });

    let mut parts = Vec::new();
    if is_calculated {
        parts.extend(boolean_option);
        parts.push(calculation);
    } else {
        parts.push(option.to_string());
        parts.extend(boolean_option);
    }

    Some(format!("{name} [ {} ]", parts.join(" ; ")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::xml_node::Node;

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
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
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
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
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
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
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
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
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
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
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
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
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
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
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
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }
}
