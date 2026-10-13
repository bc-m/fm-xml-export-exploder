use crate::utils::xml_node::{Node, Visit};

pub fn sanitize(step: &Node) -> Option<String> {
    let name = step.attr("name").unwrap_or_default();
    let mut calculation = String::new();
    let mut only_current_file = false;
    step.walk(&mut |node| match node.name.as_str() {
        "Name" => {
            only_current_file = node.attr_is_true("current");
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
    if calculation.is_empty() {
        return Some(name.to_string());
    }
    let suffix = if only_current_file {
        " ; Current file"
    } else {
        ""
    };
    Some(format!("{name} [ Name: {calculation}{suffix} ]"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::xml_node::Node;

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
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
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
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
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
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }
}
