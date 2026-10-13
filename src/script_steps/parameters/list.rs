use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::script_steps::constants::{ScriptStep, id_to_script_step};
use crate::script_steps::parameters::calculation::Calculation;
use crate::utils::attributes::parse_unescaped_attribute;

#[derive(Debug, Default)]
pub struct List {
    pub name: Option<String>,
    /// e.g. the script name calculation of "By name"
    pub calculation: Option<String>,
}

impl List {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart, step_id: u32) -> List {
        let mut depth = 1;
        let mut item = List::default();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if e.name().as_ref() == "Calculation" {
                        item.calculation = Calculation::from_xml(reader, &e).display();
                        depth -= 1;
                    } else if e.name().as_ref() == "List"
                        && let Some(name) = parse_unescaped_attribute(&e, "name")
                    {
                        item.name = match id_to_script_step(step_id) {
                            ScriptStep::LoopStart => Some(format!("Flush: {name}")),
                            _ => Some(name),
                        };
                    }
                }
                Ok(Event::End(_)) => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            buf.clear();
        }

        item
    }

    pub fn display(self) -> Option<String> {
        match (self.name, self.calculation) {
            (Some(name), Some(calculation)) => Some(format!("{name} ; {calculation}")),
            (name, calculation) => name.or(calculation),
        }
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use crate::script_steps::parameters::list::List;

    #[test]
    fn test() {
        let xml = r#"
            <Parameter type="List">
                <List id="136" name="_Home" UUID="C012F95A-8FC4-41A9-B354-2F21FE641437"></List>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "_Home".to_string();
        assert_eq!(
            List::from_xml(&mut reader, &element, 136)
                .display()
                .unwrap(),
            expected_output
        );
    }

    #[test]
    fn list_with_calculation() {
        let xml = r#"<Parameter type="List"><List name="By name" value="2"><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[Get ( ScriptName )]]></Text></Calculation></Calculation></List></Parameter>"#;
        let mut reader = Reader::from_str(xml);
        let Ok(Event::Start(e)) = reader.read_event() else {
            panic!("Wrong read event");
        };
        assert_eq!(
            List::from_xml(&mut reader, &e, 164).display().as_deref(),
            Some("By name ; Get ( ScriptName )")
        );
    }
}
