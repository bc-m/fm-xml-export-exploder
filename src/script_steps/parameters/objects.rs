//! Layout objects, media sources and menu sets.

use crate::script_steps::parameters::field_reference::FieldReference;
use crate::script_steps::parameters::format::{labeled, on_off, quoted};
use crate::utils::xml_node::Node;

/// Set Selection: `Start Position: 1 ; End Position: 12`
pub(crate) fn select(node: &Node) -> Vec<String> {
    [("Start", "Start Position"), ("End", "End Position")]
        .into_iter()
        .filter_map(|(child, label)| labeled(label, node.child(child)?.calculation()))
        .collect()
}

/// `Object Name: "name" ; Repetition: 1`
pub(crate) fn object(node: &Node) -> Vec<String> {
    [("Name", "Object Name"), ("repetition", "Repetition")]
        .into_iter()
        .filter_map(|(child, label)| labeled(label, node.child(child)?.calculation()))
        .collect()
}

/// AVPlayer Play: a layout object, a field or a URL
pub(crate) fn source(node: &Node) -> Vec<String> {
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
pub(crate) fn custom_menu_set(node: &Node) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use crate::script_steps::parameters::parameter_values::parse;

    #[test]
    fn select() {
        let xml = r#"<Parameter type="Select"><Start><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[1]]></Text></Calculation></Calculation></Start><End><Calculation datatype="1" position="1"><Calculation><Text><![CDATA[12]]></Text></Calculation></Calculation></End></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some("Start Position: 1 ; End Position: 12")
        );
    }

    #[test]
    fn object() {
        let xml = r#"<Parameter type="Object"><Name><Calculation datatype="1" position="0"><Calculation><Text><![CDATA["RepField"]]></Text></Calculation></Calculation></Name><repetition><Calculation datatype="1" position="1"><Calculation><Text><![CDATA[$Rep]]></Text></Calculation></Calculation></repetition></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some(r#"Object Name: "RepField" ; Repetition: $Rep"#)
        );
    }

    #[test]
    fn source() {
        let url = r#"<Parameter type="Source"><Source type="URL"><Calculation datatype="1" position="5"><Calculation><Text><![CDATA["http://example.com"]]></Text></Calculation></Calculation></Source></Parameter>"#;
        assert_eq!(
            parse(url, 0).as_deref(),
            Some(r#"URL: "http://example.com""#)
        );

        let field = r#"<Parameter type="Source"><Source type="FieldReference"><FieldReference id="1" name="Video"><TableOccurrenceReference id="1" name="Media"></TableOccurrenceReference><repetition><Calculation datatype="1" position="10"><Calculation><Text><![CDATA[2]]></Text></Calculation></Calculation></repetition></FieldReference></Source></Parameter>"#;
        assert_eq!(
            parse(field, 0).as_deref(),
            Some("Field: Media::Video ; Repetition: 2")
        );
    }

    #[test]
    fn custom_menu_set() {
        let xml = r#"<Parameter type="CustomMenuSet"><CustomMenuSetReference id="1" name="[Standard FileMaker Menus]"><Boolean type="Use as file default" id="512" value="False"></Boolean></CustomMenuSetReference></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some(r#""[Standard FileMaker Menus]" ; Use as file default: OFF"#)
        );
    }
}
