use crate::utils::xml_node::Node;

/// e.g. `"Script" from file "Other File" ; Parameter: $p`
pub fn display(parameter: &Node) -> Option<String> {
    let mut script_name = None;
    let mut data_source_name = None;
    // Install OnTimer Script stores its script parameter inside the reference
    let mut script_parameter = None;
    for child in parameter.descendants().into_iter().skip(1) {
        match child.name.as_str() {
            "DataSourceReference" => data_source_name = child.attr("name"),
            "ScriptReference" => script_name = child.attr("name"),
            "repetition" => script_parameter = child.calculation(),
            _ => {}
        }
    }

    let mut parts = Vec::new();
    if let Some(name) = script_name {
        parts.push(format!("\"{name}\""));
    }
    if let Some(name) = data_source_name {
        parts.push(format!("from file \"{name}\""));
    }
    let mut display = parts.join(" ");
    if let Some(script_parameter) = script_parameter {
        if !display.is_empty() {
            display.push_str(" ; ");
        }
        display.push_str(&format!("Parameter: {script_parameter}"));
    }
    (!display.is_empty()).then_some(display)
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::script_reference;

    #[test]
    fn script_with_parameter() {
        let xml = r#"<Parameter type="ScriptReference"><ScriptReference id="9" name="All script steps"><repetition><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$Param]]></Text></Calculation></Calculation></repetition></ScriptReference></Parameter>"#;
        let e = Node::parse(xml).unwrap();
        assert_eq!(
            script_reference::display(&e).as_deref(),
            Some(r#""All script steps" ; Parameter: $Param"#)
        );
    }
}
