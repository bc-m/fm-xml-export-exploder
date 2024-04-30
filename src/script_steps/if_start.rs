use crate::script_steps::parameters::calculation::Calculation;
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
                b"Step" => {
                    name = get_attribute(&e, "name").unwrap().to_string();
                }

                b"Calculation" => {
                    calculation = Calculation::from_xml(&mut reader, &e).unwrap();
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
        Some(format!("{} [ {} ]", name, calculation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize() {
        let xml_input = "
            <Step id=\"68\" name=\"Wenn\" enable=\"True\">
			<Options>16384</Options>
			<ParameterValues membercount=\"1\">
				<Parameter type=\"Calculation\">
					<Calculation datatype=\"7\" position=\"0\">
						<Calculation>
							<Text><![CDATA[Foo::Bar < Bar::Foo]]></Text>
							<ChunkList hash=\"CC82FAEEB3D0155273F45A7F1319068E\">
								<Chunk type=\"FieldRef\">
									<FieldReference id=\"4\" name=\"Bar\" repetition=\"1\">
										<TableOccurrenceReference id=\"1065089\" name=\"Foo\"></TableOccurrenceReference>
									</FieldReference>
								</Chunk>
								<Chunk type=\"NoRef\"> &lt; </Chunk>
								<Chunk type=\"FieldRef\">
									<FieldReference id=\"2\" name=\"Foo\" repetition=\"1\">
										<TableOccurrenceReference id=\"1065090\" name=\"Bar\"></TableOccurrenceReference>
									</FieldReference>
								</Chunk>
							</ChunkList>
						</Calculation>
					</Calculation>
				</Parameter>
			</ParameterValues>
		</Step>
        ";

        let expected_output = Some("Wenn [ Foo::Bar < Bar::Foo ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_sanitize_exit_loop_if() {
        let xml_input = "
		<Step id=\"72\" name=\"Verlasse Schleife wenn\" enable=\"True\">
			<SourceUUID>3ED7A361-42D8-43A8-9257-28E274A9ED6D</SourceUUID>
			<Options>16384</Options>
			<ParameterValues membercount=\"1\">
				<Parameter type=\"Calculation\">
					<Calculation datatype=\"7\" position=\"0\">
						<Calculation>
							<Text><![CDATA[$loopCount = $numberOfRelatedRecords]]></Text>
							<ChunkList hash=\"7BE699C2B6EC7CBB17F151DACAC0AFAB\">
								<Chunk type=\"VariableReference\">$loopCount</Chunk>
								<Chunk type=\"NoRef\"> = </Chunk>
								<Chunk type=\"VariableReference\">$numberOfRelatedRecords</Chunk>
							</ChunkList>
						</Calculation>
					</Calculation>
				</Parameter>
			</ParameterValues>
		</Step>
		";

        let expected_output =
            Some("Verlasse Schleife wenn [ $loopCount = $numberOfRelatedRecords ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }
}
