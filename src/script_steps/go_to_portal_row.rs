use quick_xml::Reader;
use quick_xml::events::Event;

use crate::utils::attributes::get_attribute;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut select_label = String::new();
    let mut select = false;
    let mut position = String::new();

    let mut reader = Reader::from_str(step);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => {
                    name = get_attribute(&e, "name").unwrap();
                }
                b"Boolean" => {
                    select_label = get_attribute(&e, "type").unwrap();
                    select = get_attribute(&e, "value").unwrap() == "True";
                }
                b"List" => {
                    position = get_attribute(&e, "name").unwrap();
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if name.is_empty() {
        None
    } else {
        let on_off = if select { "ON" } else { "OFF" };
        Some(format!("{name} [ {select_label}: {on_off} ; {position} ]"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
