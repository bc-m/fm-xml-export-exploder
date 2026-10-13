use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::script_steps::parameters::boolean::Boolean;
use crate::script_steps::parameters::field_reference::FieldReference;
use crate::script_steps::parameters::node::Node;

/// Parameters with nested structures, parsed into a [`Node`] tree first.
pub fn is_structured(parameter_type: &str) -> bool {
    matches!(
        parameter_type,
        "UniversalPathList"
            | "Options"
            | "Restore"
            | "Select"
            | "Object"
            | "Source"
            | "CustomMenuSet"
            | "Parameter"
            | "URL"
    )
}

pub fn from_xml(
    reader: &mut Reader<&[u8]>,
    e: &BytesStart,
    parameter_type: &str,
) -> Option<String> {
    let node = Node::from_xml(reader, e);
    let parts = match parameter_type {
        "UniversalPathList" => universal_path_list(&node),
        "Options" => options(&node),
        "Restore" => restore(&node),
        "Select" => select(&node),
        "Object" => object(&node),
        "Source" => source(&node),
        "CustomMenuSet" => custom_menu_set(&node),
        "Parameter" => parameter(&node),
        "URL" => url(&node),
        _ => Vec::new(),
    };
    Some(parts.join(" ; ")).filter(|display| !display.is_empty())
}

fn on_off(node: &Node, attribute: &str) -> &'static str {
    Boolean::on_off(node.attr_is_true(attribute))
}

/// Text that FileMaker shows inside quotes, e.g. file paths.
fn quoted(text: &str) -> String {
    format!("\"{text}\"")
}

fn labeled(label: &str, value: Option<String>) -> Option<String> {
    value.map(|value| format!("{label}: {value}"))
}

/// e.g. `"file:report.pdf" ; Automatically open ; Create email with file as attachment`
fn universal_path_list(node: &Node) -> Vec<String> {
    let Some(list) = node.child("UniversalPathList") else {
        return Vec::new();
    };
    let paths: Vec<&str> = list
        .descendant("ObjectList")
        .into_iter()
        .flat_map(|object_list| object_list.children_named("Location"))
        .map(|location| location.text.as_str())
        .collect();

    let mut parts = Vec::new();
    if !paths.is_empty() {
        parts.push(quoted(&paths.join("\n")));
    }
    if list.attr_is_true("AutoOpen") {
        parts.push("Automatically open".to_string());
    }
    if list.attr_is_true("CreateMail") {
        parts.push("Create email with file as attachment".to_string());
    }
    parts
}

fn options(node: &Node) -> Vec<String> {
    let mut parts = Vec::new();
    for options in node.children_named("Options") {
        match options.attr("type") {
            // Pause/Resume Script
            Some("Indefinitely") => parts.push("Indefinitely".to_string()),
            Some(label) if label.ends_with(": ") => {
                parts.extend(labeled(label.trim_end_matches(": "), options.calculation()));
            }
            // Insert File
            Some("Title") => parts.extend(labeled("Dialog title", options.calculation())),
            Some("Filters") => {
                let filters: Vec<String> = options
                    .descendant("Filters")
                    .into_iter()
                    .flat_map(|filters| filters.children_named("Filter"))
                    .map(|filter| {
                        let name = filter.calculation().unwrap_or_default();
                        match filter.attr("extension") {
                            Some(extension) => format!("{name} ({extension})"),
                            None => name,
                        }
                    })
                    .collect();
                if !filters.is_empty() {
                    parts.push(format!("Filters: {}", filters.join(", ")));
                }
            }
            Some(kind @ ("Storage" | "Display" | "Compress")) => {
                parts.extend(labeled(
                    kind,
                    options
                        .child(kind)
                        .and_then(|n| n.attr("name"))
                        .map(str::to_string),
                ));
            }
            // Save Records as Excel
            None => parts.extend(excel_options(options)),
            Some(_) => {}
        }
    }
    parts
}

/// e.g. `Records being browsed ; Use field names as column names: ON ; Worksheet: $Sheet`
fn excel_options(options: &Node) -> Vec<String> {
    let mut parts = Vec::new();
    if let Some(save) = options.child("Save") {
        parts.extend(save.attr("type").map(str::to_string));
        if save.attr("useFieldNames").is_some() {
            parts.push(format!(
                "Use field names as column names: {}",
                on_off(save, "useFieldNames")
            ));
        }
    }
    for parameter in options.children_named("Parameter") {
        if let Some(label) = parameter.attr("type") {
            parts.extend(labeled(label, parameter.calculation()));
        }
    }
    parts
}

fn restore(node: &Node) -> Vec<String> {
    node.child("Restore")
        .filter(|restore| restore.attr_is_true("value"))
        .map(|_| "Restore".to_string())
        .into_iter()
        .collect()
}

/// Set Selection: `Start Position: 1 ; End Position: 12`
fn select(node: &Node) -> Vec<String> {
    [("Start", "Start Position"), ("End", "End Position")]
        .into_iter()
        .filter_map(|(child, label)| labeled(label, node.child(child)?.calculation()))
        .collect()
}

/// `Object Name: "name" ; Repetition: 1`
fn object(node: &Node) -> Vec<String> {
    [("Name", "Object Name"), ("repetition", "Repetition")]
        .into_iter()
        .filter_map(|(child, label)| labeled(label, node.child(child)?.calculation()))
        .collect()
}

/// AVPlayer Play: a layout object, a field or a URL
fn source(node: &Node) -> Vec<String> {
    let Some(source) = node.child("Source") else {
        return Vec::new();
    };
    match source.attr("type") {
        Some("LayoutObject") => object(source),
        Some("FieldReference") => {
            let mut parts = vec![format!(
                "Field: {}",
                FieldReference::from_node(source).display()
            )];
            parts.extend(labeled(
                "Repetition",
                source.descendant("repetition").and_then(Node::calculation),
            ));
            parts
        }
        Some("URL") => labeled("URL", source.calculation()).into_iter().collect(),
        _ => Vec::new(),
    }
}

/// Install Menu Set: `"[Standard FileMaker Menus]" ; Use as file default: OFF`
fn custom_menu_set(node: &Node) -> Vec<String> {
    let Some(reference) = node.child("CustomMenuSetReference") else {
        return Vec::new();
    };
    let mut parts: Vec<String> = reference.attr("name").map(quoted).into_iter().collect();
    for boolean in reference.children_named("Boolean") {
        if let Some(label) = boolean.attr("type") {
            parts.push(format!("{label}: {}", on_off(boolean, "value")));
        }
    }
    parts
}

/// Perform Script on Server wraps its calculation in a nested `<Parameter>`,
/// Perform JavaScript in Web Viewer lists one calculation per function parameter.
fn parameter(node: &Node) -> Vec<String> {
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
fn url(node: &Node) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use super::*;

    fn parse(xml: &str) -> Option<String> {
        let mut reader = Reader::from_str(xml);
        let Ok(Event::Start(e)) = reader.read_event() else {
            panic!("Wrong read event");
        };
        let parameter_type = crate::utils::attributes::get_attribute(&e, "type").unwrap();
        from_xml(&mut reader, &e, &parameter_type)
    }

    #[test]
    fn universal_path_list() {
        let xml = r#"<Parameter type="UniversalPathList"><UniversalPathList AutoOpen="True" CreateMail="True" membercount="2"><ObjectList><Location><![CDATA[file:a.txt]]></Location><Location><![CDATA[$Path]]></Location></ObjectList></UniversalPathList></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some(
                "\"file:a.txt\n$Path\" ; Automatically open ; Create email with file as attachment"
            )
        );
    }

    #[test]
    fn pause_options() {
        let indefinitely =
            r#"<Parameter type="Options"><Options type="Indefinitely"></Options></Parameter>"#;
        assert_eq!(parse(indefinitely).as_deref(), Some("Indefinitely"));

        let duration = r#"<Parameter type="Options"><Options type="Duration (seconds): "><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[1]]></Text></Calculation></Calculation></Options></Parameter>"#;
        assert_eq!(parse(duration).as_deref(), Some("Duration (seconds): 1"));
    }

    #[test]
    fn insert_file_options() {
        let xml = r#"<Parameter type="Options"><Options type="Title"><Title><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$Title]]></Text></Calculation></Calculation></Title></Options><Options type="Filters"><Filters size="1"><Filter extension="*"><Calculation><Text><![CDATA["All Files"]]></Text></Calculation></Filter></Filters></Options><Options type="Storage"><Storage name="Let user choose" value="0"></Storage></Options><Options type="Display"><Display name="Display icon" value="0"></Display></Options><Options type="Compress"><Compress name="Never compress" value="1"></Compress></Options></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some(
                r#"Dialog title: $Title ; Filters: "All Files" (*) ; Storage: Let user choose ; Display: Display icon ; Compress: Never compress"#
            )
        );
    }

    #[test]
    fn excel_options() {
        let xml = r#"<Parameter type="Options"><Options><Save type="Records being browsed" value="1" useFieldNames="True"></Save><Parameter type="Worksheet"><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$WKS1]]></Text></Calculation></Calculation></Parameter><Parameter type="Title"><Calculation datatype="1" position="1"><Calculation><Text><![CDATA[$Title]]></Text></Calculation></Calculation></Parameter></Options></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some(
                "Records being browsed ; Use field names as column names: ON ; Worksheet: $WKS1 ; Title: $Title"
            )
        );
    }

    #[test]
    fn empty_options() {
        let xml = r#"<Parameter type="Options"><Options></Options></Parameter>"#;
        assert_eq!(parse(xml), None);
    }

    #[test]
    fn restore() {
        let on = r#"<Parameter type="Restore"><Restore value="True"></Restore></Parameter>"#;
        let off = r#"<Parameter type="Restore"><Restore value="False"></Restore></Parameter>"#;
        assert_eq!(parse(on).as_deref(), Some("Restore"));
        assert_eq!(parse(off), None);
    }

    #[test]
    fn select() {
        let xml = r#"<Parameter type="Select"><Start><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[1]]></Text></Calculation></Calculation></Start><End><Calculation datatype="1" position="1"><Calculation><Text><![CDATA[12]]></Text></Calculation></Calculation></End></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some("Start Position: 1 ; End Position: 12")
        );
    }

    #[test]
    fn object() {
        let xml = r#"<Parameter type="Object"><Name><Calculation datatype="1" position="0"><Calculation><Text><![CDATA["RepField"]]></Text></Calculation></Calculation></Name><repetition><Calculation datatype="1" position="1"><Calculation><Text><![CDATA[$Rep]]></Text></Calculation></Calculation></repetition></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some(r#"Object Name: "RepField" ; Repetition: $Rep"#)
        );
    }

    #[test]
    fn source() {
        let url = r#"<Parameter type="Source"><Source type="URL"><Calculation datatype="1" position="5"><Calculation><Text><![CDATA["http://example.com"]]></Text></Calculation></Calculation></Source></Parameter>"#;
        assert_eq!(parse(url).as_deref(), Some(r#"URL: "http://example.com""#));

        let field = r#"<Parameter type="Source"><Source type="FieldReference"><FieldReference id="1" name="Video"><TableOccurrenceReference id="1" name="Media"></TableOccurrenceReference><repetition><Calculation datatype="1" position="10"><Calculation><Text><![CDATA[2]]></Text></Calculation></Calculation></repetition></FieldReference></Source></Parameter>"#;
        assert_eq!(
            parse(field).as_deref(),
            Some("Field: Media::Video ; Repetition: 2")
        );
    }

    #[test]
    fn custom_menu_set() {
        let xml = r#"<Parameter type="CustomMenuSet"><CustomMenuSetReference id="1" name="[Standard FileMaker Menus]"><Boolean type="Use as file default" id="512" value="False"></Boolean></CustomMenuSetReference></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some(r#""[Standard FileMaker Menus]" ; Use as file default: OFF"#)
        );
    }

    #[test]
    fn parameter() {
        let nested = r#"<Parameter type="Parameter"><Parameter><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$p]]></Text></Calculation></Calculation></Parameter></Parameter>"#;
        assert_eq!(parse(nested).as_deref(), Some("Parameter: $p"));

        let empty = r#"<Parameter type="Parameter"><Parameter></Parameter></Parameter>"#;
        assert_eq!(parse(empty), None);

        let multiple = r#"<Parameter type="Parameter"><Calculation datatype="1" position="10"><Calculation><Text><![CDATA[$p1]]></Text></Calculation></Calculation><Calculation datatype="1" position="11"><Calculation><Text><![CDATA[$p2]]></Text></Calculation></Calculation></Parameter>"#;
        assert_eq!(parse(multiple).as_deref(), Some("Parameters: $p1 ; $p2"));
    }

    #[test]
    fn url() {
        let xml = r#"<Parameter type="URL"><URL autoEncode="False"><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$url]]></Text></Calculation></Calculation></URL></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some("URL: $url ; Automatically encode URL: OFF")
        );
    }
}
