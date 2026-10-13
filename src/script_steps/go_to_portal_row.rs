use crate::script_steps::parameters::boolean::Boolean;
use crate::utils::xml_node::{Node, Visit};

pub fn sanitize(step: &Node) -> Option<String> {
    let name = step.attr("name").unwrap_or_default();
    let mut select_label = "";
    let mut select = false;
    let mut position = "";
    step.walk(&mut |node| {
        match node.name.as_str() {
            "Boolean" => {
                select_label = node.attr("type").unwrap_or_default();
                select = node.attr_is_true("value");
            }
            "List" => position = node.attr("name").unwrap_or_default(),
            _ => {}
        }
        Visit::Descend
    });

    if name.is_empty() {
        return None;
    }
    Some(format!(
        "{name} [ {select_label}: {} ; {position} ]",
        Boolean::on_off(select)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::xml_node::Node;

    #[test]
    fn test() {
        let xml = r#"
            <Step id="99" name="Gehe zu Ausschnittreihe" enable="True">
                <SourceUUID>1574DCD5-7ECE-4CE0-A443-E20F4770DC86</SourceUUID>
                <Options>4098</Options>
                <ParameterValues membercount="2">
                    <Parameter type="Boolean">
                        <Boolean type="Auswahl" id="4096" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="Portal">
                        <List name="Letzte(r)" value="2"></List>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some("Gehe zu Ausschnittreihe [ Auswahl: ON ; Letzte(r) ]".to_string());
        assert_eq!(sanitize(&Node::parse(xml).unwrap()), expected_output);
    }
}
