//! Script parameters, URLs, SQL, variables and Apple events.

use crate::script_steps::parameters::files::universal_path_list;
use crate::script_steps::parameters::format::{labeled, on_off, quoted};
use crate::utils::xml_node::Node;

/// Perform Script on Server wraps its calculation in a nested `<Parameter>`,
/// Perform JavaScript in Web Viewer lists one calculation per function parameter.
pub(crate) fn parameter(node: &Node) -> Vec<String> {
    let calculations: Vec<String> = match node.child("Parameter") {
        Some(nested) => nested.calculation().into_iter().collect(),
        None => node
            .children_named("Calculation")
            .filter_map(Node::calculation)
            .collect(),
    };
    match calculations.len() {
        0 => Vec::new(),
        1 => vec![format!("Parameter: {}", calculations[0])],
        _ => vec![format!("Parameters: {}", calculations.join(" ; "))],
    }
}

/// `URL: $url ; Automatically encode URL: ON`
pub(crate) fn url(node: &Node) -> Vec<String> {
    let Some(url) = node.child("URL") else {
        return Vec::new();
    };
    let Some(calculation) = url.calculation() else {
        return Vec::new();
    };
    vec![
        format!("URL: {calculation}"),
        format!("Automatically encode URL: {}", on_off(url, "autoEncode")),
    ]
}

/// Execute SQL: `ODBC: dsn ; SQL Text: SELECT ...`
pub(crate) fn sql(node: &Node) -> Vec<String> {
    let Some(sql) = node.child("SQL") else {
        return Vec::new();
    };
    let mut parts = Vec::new();
    if let Some(dsn) = sql.child("DSN").and_then(|d| d.attr("name"))
        && !dsn.is_empty()
    {
        parts.push(format!("ODBC: {dsn}"));
    }
    parts.extend(labeled("SQL Text", sql.calculation()));
    parts
}

/// Target variable, e.g. `Target: $path` or `Target: $path[2]`
pub(crate) fn variable(node: &Node) -> Vec<String> {
    let Some(name) = node
        .child("Name")
        .and_then(|n| n.attr("value"))
        .filter(|n| !n.is_empty())
    else {
        return Vec::new();
    };
    let repetition = node.child("repetition").and_then(Node::calculation);
    match repetition.as_deref() {
        Some(rep) if rep != "1" => vec![format!("Target: {name}[{rep}]")],
        _ => vec![format!("Target: {name}")],
    }
}

/// Send Event: e.g. `Target application: "iCal.app" ; Event: open application ;
/// Wait for event completion: ON`
pub(crate) fn send_event(node: &Node) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(target) = node.child("Target").and_then(|t| t.attr("value"))
        && !target.is_empty()
    {
        parts.push(format!("Target application: {}", quoted(target)));
    }
    if let Some(send) = node.child("Send") {
        let event = send.attr("name").map(str::to_string).or_else(|| {
            // Events without a name are identified by their class and id codes
            let class = send.attr("classString").filter(|c| *c != "????")?;
            let id = send.attr("idString").filter(|i| *i != "????")?;
            Some(format!("{class}/{id}"))
        });
        parts.extend(labeled("Event", event));
    }
    if let Some(source) = node.child("Source") {
        match source.attr("type") {
            Some("Document") => {
                let paths = source.child("Document").map(universal_path_list);
                if let Some(path) = paths.as_ref().and_then(|p| p.first()) {
                    parts.push(format!("Document: {path}"));
                }
            }
            Some("Text") => {
                let text = source.child("Text").and_then(|text| {
                    text.calculation()
                        .or_else(|| Some(text.text().to_string()).filter(|t| !t.is_empty()))
                });
                parts.extend(labeled("Text", text));
            }
            Some("Calculation") => parts.extend(labeled("Calculation", source.calculation())),
            _ => {}
        }
    }
    if let Some(options) = node.child("Options") {
        for (attribute, label) in [
            ("Foreground", "Bring target application to foreground"),
            ("Wait", "Wait for event completion"),
            ("Copy", "Copy event result"),
        ] {
            if options.attr(attribute).is_some() {
                parts.push(format!("{label}: {}", on_off(options, attribute)));
            }
        }
    }
    Some(parts.join(" ; ")).filter(|display| !display.is_empty())
}

/// Trigger Claris Connect Flow, e.g. `Flow: <unknown>`
pub(crate) fn flow(node: &Node) -> Vec<String> {
    labeled(
        "Flow",
        node.child("Flow")
            .and_then(|flow| flow.attr("value"))
            .map(str::to_string),
    )
    .into_iter()
    .collect()
}

#[cfg(test)]
mod tests {
    use crate::script_steps::parameters::parameter_values::parse;

    #[test]
    fn parameter() {
        let nested = r#"<Parameter type="Parameter"><Parameter><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$p]]></Text></Calculation></Calculation></Parameter></Parameter>"#;
        assert_eq!(parse(nested, 0).as_deref(), Some("Parameter: $p"));

        let empty = r#"<Parameter type="Parameter"><Parameter></Parameter></Parameter>"#;
        assert_eq!(parse(empty, 0), None);

        let multiple = r#"<Parameter type="Parameter"><Calculation datatype="1" position="10"><Calculation><Text><![CDATA[$p1]]></Text></Calculation></Calculation><Calculation datatype="1" position="11"><Calculation><Text><![CDATA[$p2]]></Text></Calculation></Calculation></Parameter>"#;
        assert_eq!(parse(multiple, 0).as_deref(), Some("Parameters: $p1 ; $p2"));
    }

    #[test]
    fn url() {
        let xml = r#"<Parameter type="URL"><URL autoEncode="False"><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$url]]></Text></Calculation></Calculation></URL></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some("URL: $url ; Automatically encode URL: OFF")
        );
    }

    #[test]
    fn sql() {
        let xml = r#"<Parameter type="SQL"><SQL suppress="False" DSN="False"><DSN name="" UserName="" Password=""></DSN><Calculation><Text><![CDATA[SELECT * FROM orders]]></Text></Calculation></SQL></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some("SQL Text: SELECT * FROM orders")
        );
    }

    #[test]
    fn variable() {
        let xml = r#"<Parameter type="Variable"><value></value><Name value="$var"></Name><repetition><Calculation datatype="1" position="2"><Calculation><Text><![CDATA[2]]></Text></Calculation></Calculation></repetition></Parameter>"#;
        assert_eq!(parse(xml, 0).as_deref(), Some("Target: $var[2]"));
    }

    #[test]
    fn send_event() {
        let xml = r#"<Parameter><Target value="TextMate.app" type="TxMt" id="1"></Target><Send name="open document" classString="aevt" idString="odoc" class="1" id="2"></Send><Source type="Document"><Document><UniversalPathList membercount="1"><ObjectList><Location><![CDATA[file:a.xml]]></Location></ObjectList></UniversalPathList></Document></Source><Options Foreground="True" Wait="True" Copy="False"></Options></Parameter>"#;
        assert_eq!(
            parse(xml, 57).as_deref(),
            Some(
                r#"Target application: "TextMate.app" ; Event: open document ; Document: "file:a.xml" ; Bring target application to foreground: ON ; Wait for event completion: ON ; Copy event result: OFF"#
            )
        );
    }
}
