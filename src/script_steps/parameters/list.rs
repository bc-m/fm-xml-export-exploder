use crate::script_steps::constants::{ScriptStep, id_to_script_step};
use crate::utils::xml_node::Node;

/// The selected option of a list, e.g. `Next`, plus a nested calculation if any,
/// e.g. `By name ; Get ( ScriptName )`
pub fn display(parameter: &Node, step_id: u32) -> Option<String> {
    let mut name = None;
    let mut calculation = None;
    for child in parameter.descendants().into_iter().skip(1) {
        match child.name.as_str() {
            "Calculation" => calculation = child.calculation(),
            "List" => {
                if let Some(list_name) = child.attr("name") {
                    name = Some(match id_to_script_step(step_id) {
                        ScriptStep::LoopStart => format!("Flush: {list_name}"),
                        _ => list_name.to_string(),
                    });
                }
            }
            _ => {}
        }
    }
    match (name, calculation) {
        (Some(name), Some(calculation)) => Some(format!("{name} ; {calculation}")),
        (name, calculation) => name.or(calculation),
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::list;

    #[test]
    fn test() {
        let xml = r#"
            <Parameter type="List">
                <List id="136" name="_Home" UUID="C012F95A-8FC4-41A9-B354-2F21FE641437"></List>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();

        let expected_output = "_Home".to_string();
        assert_eq!(list::display(&element, 136).unwrap(), expected_output);
    }

    #[test]
    fn list_with_calculation() {
        let xml = r#"<Parameter type="List"><List name="By name" value="2"><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[Get ( ScriptName )]]></Text></Calculation></Calculation></List></Parameter>"#;
        let e = Node::parse(xml).unwrap();
        assert_eq!(
            list::display(&e, 164).as_deref(),
            Some("By name ; Get ( ScriptName )")
        );
    }
}
