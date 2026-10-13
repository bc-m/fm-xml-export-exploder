use crate::utils::xml_node::Node;

/// A file reference (e.g. `"Other File"`) and, for Convert File and Import Records,
/// the XML data source, e.g. `XML: "http:xml" ; XSL: "$path"`
pub fn display(parameter: &Node) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(reference) = parameter.last_descendant("DataSourceReference")
        && let Some(name) = reference.attr("name")
    {
        parts.push(if reference.attr("id") == Some("0") {
            name.to_string()
        } else {
            format!("\"{name}\"")
        });
    }
    for source in parameter
        .descendants()
        .into_iter()
        .filter(|n| matches!(n.name.as_str(), "XML" | "XSL"))
    {
        let value = source.calculation().or_else(|| {
            let paths: Vec<&str> = source
                .descendant("ObjectList")?
                .children_named("Location")
                .map(Node::text)
                .collect();
            (!paths.is_empty()).then(|| format!("\"{}\"", paths.join("\n")))
        });
        if let Some(value) = value {
            parts.push(format!("{}: {value}", source.name));
        }
    }
    (!parts.is_empty()).then(|| parts.join(" ; "))
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::data_source_reference;

    #[test]
    fn xml_source() {
        let xml = r#"<Parameter type="DataSourceReference"><DataSourceReference fileType="XML " isPredefined="False" delimiter="4" value="4"><XML><Calculation><Text><![CDATA["http:xml"]]></Text></Calculation></XML><XSL><UniversalPathList fileType="XML " membercount="1"><ObjectList><Location><![CDATA[$PathXSLT]]></Location></ObjectList></UniversalPathList></XSL></DataSourceReference></Parameter>"#;
        let e = Node::parse(xml).unwrap();
        assert_eq!(
            data_source_reference::display(&e).as_deref(),
            Some(r#"XML: "http:xml" ; XSL: "$PathXSLT""#)
        );
    }
}
