use crate::script_steps::parameters::boolean::Boolean;
use crate::utils::xml_node::{Node, Visit};

pub fn sanitize(step: &Node) -> Option<String> {
    let name = step.attr("name").unwrap_or_default();
    let mut option_name = "";
    let mut state = false;
    let mut calculation = String::new();
    step.walk(&mut |node| match node.name.as_str() {
        "Boolean" => {
            option_name = node.attr("type").unwrap_or_default();
            state = node.attr_is_true("value");
            Visit::Descend
        }
        "Calculation" => {
            calculation = node.calculation().unwrap_or_default();
            Visit::Skip
        }
        _ => Visit::Descend,
    });

    if option_name.is_empty() && calculation.is_empty() {
        return Some(format!("{name} []"));
    }

    let mut parts = vec![format!("{option_name}: {}", Boolean::on_off(state))];
    if !calculation.is_empty() {
        parts.push(calculation);
    }
    Some(format!("{name} [ {} ]", parts.join(" ; ")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::xml_node::Node;

    #[test]
    fn test() {
        let xml = r#"
            <Step id="26" name="Mehrere ausschließen" enable="True">
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="True"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Mehrere ausschließen [ Mit Dialog: ON ]".to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }

    #[test]
    fn test_with_calculation() {
        let xml = r#"
            <Step id="26" name="Mehrere ausschließen" enable="True">
                <Options>16512</Options>
                <ParameterValues membercount="2">
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Calculation">
                        <Calculation datatype="1" position="0">
                            <Calculation>
                                <Text><![CDATA[123]]></Text>
                                <ChunkList hash="90213DC459B04C5A47C044B9460AEF7B">
                                    <Chunk type="NoRef">123</Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Mehrere ausschließen [ Mit Dialog: OFF ; 123 ]".to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }
}
