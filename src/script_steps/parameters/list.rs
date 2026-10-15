use crate::script_steps::constants::{ScriptStep, id_to_script_step};
use crate::script_steps::parameters::script_reference;
use crate::utils::xml_node::Node;

/// The selected option of a list, e.g. `Next`, plus a nested calculation if any,
/// e.g. `By name ; Get ( ScriptName )`, or the script selected from a list, e.g.
/// `From list ; "Script"`
pub fn display(parameter: &Node, step_id: u32) -> Option<String> {
    let mut name = None;
    let mut calculation = None;
    let mut has_script_reference = false;
    for child in parameter.descendants().into_iter().skip(1) {
        match child.name.as_str() {
            "Calculation" => calculation = child.calculation(),
            "ScriptReference" => has_script_reference = true,
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
    if has_script_reference && let Some(script) = script_reference::display(parameter) {
        return Some(match name {
            Some(name) => format!("{name} ; {script}"),
            None => script,
        });
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
    fn list_with_script_reference() {
        let xml = r#"<Parameter type="List"><List name="From list" value="1"><ScriptReference id="787" name="Scrape"></ScriptReference></List></Parameter>"#;
        let e = Node::parse(xml).unwrap();
        assert_eq!(
            list::display(&e, 164).as_deref(),
            Some(r#"From list ; "Scrape""#)
        );
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
