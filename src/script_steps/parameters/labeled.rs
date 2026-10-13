use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::script_steps::constants::{ScriptStep, id_to_script_step};
use crate::script_steps::parameters::boolean::Boolean;
use crate::script_steps::parameters::calculation::Calculation;
use crate::script_steps::parameters::list::List;
use crate::utils::xml_utils::skip_rest_of_element;

/// Returns the label FileMaker shows for parameters that wrap a single
/// `<Calculation>`, `<Boolean>` or `<List>` element.
pub fn label(parameter_type: &str, step_id: u32) -> Option<&'static str> {
    Some(match parameter_type {
        "id" => "File ID",
        "size" => "Amount (bytes)",
        "position" if id_to_script_step(step_id) == ScriptStep::SetDataFilePosition => {
            "New position"
        }
        "position" => "Position",
        "UUID" => "UUID",
        "Major" => "Major",
        "Minor" => "Minor",
        "Latitude" => "Latitude",
        "Longitude" => "Longitude",
        "Radius" => "Radius",
        "End" => "End",
        "Volume" => "Volume",
        "Location" => "Default location",
        "FunctionRef" => "Function Name",
        "Monitor" => "Monitor",
        "Presentation" => "Presentation",
        "action" => "Action",
        "perform" => "Perform",
        "direction" => "Direction",
        "across" => "Across",
        "within" => "Within",
        "Zoom" => "Zoom",
        "Sequence" => "Sequence",
        "Hide" => "Hide",
        "DisableInteraction" => "Disable interaction",
        "DisableExternalControls" => "Disable external controls",
        "PauseInBackground" => "Pause in background",
        // Booleans whose `type` attribute already holds the display label
        "Start" | "Wait" | "case" | "whole" => "",
        _ => return None,
    })
}

/// Parses a parameter whose value is a single wrapped `<Calculation>`, `<Boolean>`
/// or `<List>` element and displays it as `label: value`.
pub fn from_xml(
    reader: &mut Reader<&[u8]>,
    e: &BytesStart,
    step_id: u32,
    label: &str,
) -> Option<String> {
    match first_child_name(reader).as_deref() {
        Some("Calculation") => Calculation::from_xml(reader, e)
            .display()
            .map(|value| format!("{label}: {value}")),
        Some("Boolean") => {
            let mut boolean = Boolean::from_xml(reader, e, step_id);
            if !label.is_empty() {
                boolean.name = Some(label.to_string());
            }
            boolean.display()
        }
        Some("List") => List::from_xml(reader, e, step_id)
            .display()
            .map(|value| format!("{label}: {value}")),
        _ => {
            skip_rest_of_element(reader);
            None
        }
    }
}

/// Peeks at the name of the first child element without consuming any events.
fn first_child_name(reader: &Reader<&[u8]>) -> Option<String> {
    let mut peek = reader.clone();
    let mut buf = Vec::new();
    loop {
        match peek.read_event_into(&mut buf) {
            Ok(Event::Start(child)) | Ok(Event::Empty(child)) => {
                return Some(child.name().as_ref().to_owned());
            }
            Ok(Event::End(_)) | Ok(Event::Eof) | Err(_) => return None,
            _ => {}
        }
        buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use super::*;

    fn parse(xml: &str, step_id: u32) -> Option<String> {
        let mut reader = Reader::from_str(xml.trim());
        let Ok(Event::Start(e)) = reader.read_event() else {
            panic!("Wrong read event");
        };
        let parameter_type = crate::utils::attributes::get_attribute(&e, "type").unwrap();
        let label = label(&parameter_type, step_id).unwrap();
        from_xml(&mut reader, &e, step_id, label)
    }

    #[test]
    fn calculation() {
        let xml = r#"<Parameter type="size"><Calculation datatype="2" position="1"><Calculation><Text><![CDATA[123]]></Text></Calculation></Calculation></Parameter>"#;
        assert_eq!(parse(xml, 193).as_deref(), Some("Amount (bytes): 123"));
    }

    #[test]
    fn position_depends_on_step() {
        let xml = r#"<Parameter type="position"><Calculation datatype="2" position="1"><Calculation><Text><![CDATA[$Pos]]></Text></Calculation></Calculation></Parameter>"#;
        assert_eq!(parse(xml, 195).as_deref(), Some("New position: $Pos"));
        assert_eq!(parse(xml, 177).as_deref(), Some("Position: $Pos"));
    }

    #[test]
    fn list() {
        let xml = r#"<Parameter type="Monitor"><List name="iBeacon" value="0"></List></Parameter>"#;
        assert_eq!(parse(xml, 185).as_deref(), Some("Monitor: iBeacon"));
    }

    #[test]
    fn boolean_with_label() {
        let xml = r#"<Parameter type="DisableInteraction"><Boolean type="DisableInteraction" value="True"></Boolean></Parameter>"#;
        assert_eq!(parse(xml, 177).as_deref(), Some("Disable interaction: ON"));
    }

    #[test]
    fn boolean_keeps_own_type() {
        let xml = r#"<Parameter type="case"><Boolean type="Match case" value="False"></Boolean></Parameter>"#;
        assert_eq!(parse(xml, 128).as_deref(), Some("Match case: OFF"));
    }

    #[test]
    fn empty_parameter() {
        let xml = r#"<Parameter type="find"></Parameter><Parameter type="next"></Parameter>"#;
        let mut reader = Reader::from_str(xml);
        let Ok(Event::Start(e)) = reader.read_event() else {
            panic!("Wrong read event");
        };
        assert_eq!(from_xml(&mut reader, &e, 128, "Find"), None);
        // The following sibling must still be readable
        let Ok(Event::Start(next)) = reader.read_event() else {
            panic!("Sibling consumed");
        };
        assert_eq!(next.name().as_ref(), "Parameter");
    }
}
