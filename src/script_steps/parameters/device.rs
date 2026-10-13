//! Insert from Device and the Speak voice.

use crate::script_steps::parameters::field_reference::FieldReference;
use crate::script_steps::parameters::format::{labeled, on_off};
use crate::utils::xml_node::Node;

/// Insert from Device: the source (e.g. Camera, Barcode, Signature) and its options,
/// e.g. `Camera ; Camera: Back ; Light mode: Auto ; Resolution: Full`
pub(crate) fn device_source(node: &Node) -> Option<String> {
    let list = node.child("List")?;
    let mut parts: Vec<String> = list.attr("name").map(str::to_string).into_iter().collect();
    for options in list.children_named("Options") {
        for parameter in options.children_named("Parameter") {
            parts.extend(device_option(parameter));
        }
    }
    Some(parts.join(" ; ")).filter(|display| !display.is_empty())
}

fn device_option(parameter: &Node) -> Option<String> {
    let kind = parameter.attr("type")?;
    let label = match kind {
        "LightMode" => "Light mode",
        "type" => "Barcode types",
        "FieldReference" => "Barcode types from field",
        other => other,
    };

    if kind == "FieldReference" {
        return Some(format!(
            "{label}: {}",
            FieldReference::from_node(parameter).display()
        ));
    }
    if let Some(list) = parameter.child("List") {
        // Barcode types list one <name> per selected type
        let names: Vec<&str> = list
            .children_named("name")
            .map(|name| name.text())
            .collect();
        let value = if names.is_empty() {
            list.attr("name")?.to_string()
        } else {
            names.join(", ")
        };
        return Some(format!("{label}: {value}"));
    }
    if let Some(boolean) = parameter.child("Boolean") {
        let boolean_label = boolean.attr("type").unwrap_or(label);
        // Max duration carries its value as a calculation next to the Boolean
        return match parameter.calculation() {
            Some(value) if boolean.attr_is_true("value") => {
                Some(format!("{boolean_label}: {value}"))
            }
            Some(_) => None,
            None => Some(format!("{boolean_label}: {}", on_off(boolean, "value"))),
        };
    }
    labeled(label, parameter.calculation())
}

/// Speak (macOS): the voice is stored as a synthesizer creator code (a FourCC like
/// `gala`) and a voice id, without the voice name. `0`/`0` is the default voice.
pub(crate) fn voice(node: &Node) -> Vec<String> {
    let Some(voice) = node.child("Voice") else {
        return Vec::new();
    };
    let id = voice.attr("id").unwrap_or("0");
    let creator = voice.attr("value").unwrap_or("0");
    if id == "0" && creator == "0" {
        return vec!["Voice: Default".to_string()];
    }
    let creator = creator
        .parse::<u32>()
        .ok()
        .map(u32::to_be_bytes)
        .filter(|bytes| bytes.iter().all(u8::is_ascii_graphic))
        .map_or_else(
            || creator.to_string(),
            |bytes| String::from_utf8_lossy(&bytes).into_owned(),
        );
    vec![format!("Voice: {creator} {id}")]
}

#[cfg(test)]
mod tests {
    use crate::script_steps::parameters::parameter_values::parse;

    #[test]
    fn device_camera() {
        let xml = r#"<Parameter type="List"><List name="Camera" value="3"><Options type="Camera"><Parameter type="Camera"><List name="Back" value="1"></List></Parameter><Parameter type="LightMode"><List name="Auto" value="0"></List></Parameter><Parameter type="Resolution"><List name="Medium" value="2"></List></Parameter></Options></List></Parameter>"#;
        assert_eq!(
            parse(xml, 161).as_deref(),
            Some("Camera ; Camera: Back ; Light mode: Auto ; Resolution: Medium")
        );
    }

    #[test]
    fn device_microphone() {
        let xml = r#"<Parameter type="List"><List name="Microphone" value="5"><Options type="Microphone"><Parameter type="Duration"><Boolean value="True" type="Max duration"></Boolean><Calculation datatype="2" position="0"><Calculation><Text><![CDATA[12]]></Text></Calculation></Calculation></Parameter><Parameter type="Start"><Boolean value="True" type="Start immediately"></Boolean></Parameter></Options></List></Parameter>"#;
        assert_eq!(
            parse(xml, 161).as_deref(),
            Some("Microphone ; Max duration: 12 ; Start immediately: ON")
        );
    }

    #[test]
    fn device_barcode() {
        let xml = r#"<Parameter type="List"><List name="Barcode" value="6"><Options type="Barcode"><Parameter type="type"><List value="3"><name>Code 39</name><name>QR Code</name></List></Parameter></Options></List></Parameter>"#;
        assert_eq!(
            parse(xml, 161).as_deref(),
            Some("Barcode ; Barcode types: Code 39, QR Code")
        );
    }

    #[test]
    fn device_signature() {
        let xml = r#"<Parameter type="List"><List name="Signature" value="7"><Options type="Signature"><Parameter type="Title"><Calculation datatype="1" position="0"><Calculation><Text><![CDATA["Title"]]></Text></Calculation></Calculation></Parameter><Parameter type="Prompt"><Calculation datatype="1" position="1"><Calculation><Text><![CDATA["Sign here"]]></Text></Calculation></Calculation></Parameter><Parameter type="Presentation"><List name="Overlay" value="1"></List></Parameter></Options></List></Parameter>"#;
        assert_eq!(
            parse(xml, 161).as_deref(),
            Some(r#"Signature ; Title: "Title" ; Prompt: "Sign here" ; Presentation: Overlay"#)
        );
    }

    #[test]
    fn device_without_options() {
        let xml =
            r#"<Parameter type="List"><List name="Music Library" value="1"></List></Parameter>"#;
        assert_eq!(parse(xml, 161).as_deref(), Some("Music Library"));
    }

    #[test]
    fn voice() {
        let default = r#"<Parameter type="Voice"><Voice id="0" value="0"></Voice></Parameter>"#;
        assert_eq!(parse(default, 0).as_deref(), Some("Voice: Default"));

        let agnes =
            r#"<Parameter type="Voice"><Voice id="300" value="1734437985"></Voice></Parameter>"#;
        assert_eq!(parse(agnes, 0).as_deref(), Some("Voice: gala 300"));

        let unprintable = r#"<Parameter type="Voice"><Voice id="7" value="1"></Voice></Parameter>"#;
        assert_eq!(parse(unprintable, 0).as_deref(), Some("Voice: 1 7"));
    }
}
