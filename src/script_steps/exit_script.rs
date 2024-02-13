use crate::calculations::calculation::Calculation;
use crate::utils::attributes::get_attribute;
use quick_xml::events::Event;
use quick_xml::Reader;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut calculation = String::new();

    let mut reader = Reader::from_str(step);
    reader.trim_text(true);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => name = get_attribute(&e, "name").unwrap().to_string(),
                b"Calculation" => calculation = Calculation::from_xml(&mut reader, &e).unwrap(),
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if name.is_empty() {
        None
    } else if calculation.is_empty() {
        Some(format!("{} []", name))
    } else {
        Some(format!("{} [ {} ]", name, calculation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_without_calculation() {
        let xml_input =
            "<Step id=\"103\" name=\"Aktuelles Script verlassen\" enable=\"True\"></Step>";

        let expected_output = Some("Aktuelles Script verlassen []".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_sanitize_with_calculation() {
        let xml_input = "
		<Step id=\"103\" name=\"Aktuelles Script verlassen\" enable=\"True\">
			<Options>16384</Options>
			<ParameterValues membercount=\"1\">
				<Parameter type=\"Calculation\">
					<Calculation datatype=\"1\" position=\"0\">
						<Calculation>
							<Text><![CDATA[$Foo]]></Text>
							<ChunkList hash=\"3FC07F05CB535D7FBE7DBB56621B41CB\">
								<Chunk type=\"VariableReference\">$Foo</Chunk>
							</ChunkList>
						</Calculation>
					</Calculation>
				</Parameter>
			</ParameterValues>
		</Step>
        ";

        let expected_output = Some("Aktuelles Script verlassen [ $Foo ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }
}
