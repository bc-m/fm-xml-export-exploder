use crate::calculations::layout_reference::LayoutReferenceContainer;
use crate::utils::attributes::get_attribute;
use quick_xml::events::Event;
use quick_xml::Reader;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut animation = String::new();
    let mut layout_reference = String::new();

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
                b"LayoutReferenceContainer" => {
                    layout_reference = LayoutReferenceContainer::from_xml(&mut reader, &e)
                        .unwrap()
                        .display()
                        .unwrap();
                }
                b"Animation" => {
                    animation = get_attribute(&e, "name").unwrap().to_string();
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
        Some(format!(
            "{} [ {} ; Animation: {} ]",
            name, layout_reference, animation
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize() {
        let xml_input = "
        <Step index=\"5\" id=\"6\" name=\"Gehe zu Layout\" enable=\"True\">
            <UUID>2862164E-6771-4BF9-904A-81C64E5B38F2</UUID>
            <OwnerID></OwnerID>
            <Options>0</Options>
            <ParameterValues membercount=\"2\">
                <Parameter type=\"LayoutReferenceContainer\">
                    <LayoutReferenceContainer value=\"1\">
                        <Label>Originallayout</Label>
                    </LayoutReferenceContainer>
                </Parameter>
                <Parameter type=\"Animation\">
                    <Animation name=\"Ohne\" value=\"0\"></Animation>
                </Parameter>
            </ParameterValues>
        </Step>
        ";

        let expected_output =
            Some("Gehe zu Layout [ Layout: <Originallayout> ; Animation: Ohne ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_sanitize_layout() {
        let xml_input = "
        <Step index=\"6\" id=\"6\" name=\"Gehe zu Layout\" enable=\"True\">
            <UUID>FF4782F3-0CC1-476B-ACA0-51A60F932210</UUID>
            <OwnerID></OwnerID>
            <Options>8</Options>
            <ParameterValues membercount=\"2\">
                <Parameter type=\"LayoutReferenceContainer\">
                    <LayoutReferenceContainer value=\"5\">
                        <LayoutReference id=\"1\" name=\"FooLayout\" UUID=\"A8376443-64B0-4A0B-B44D-EB9C770B0DA7\"></LayoutReference>
                    </LayoutReferenceContainer>
                </Parameter>
                <Parameter type=\"Animation\">
                    <Animation name=\"Ohne\" value=\"0\"></Animation>
                </Parameter>
            </ParameterValues>
        </Step>
        ";

        let expected_output =
            Some("Gehe zu Layout [ Layout: \"FooLayout\" ; Animation: Ohne ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_sanitize_by_name() {
        let xml_input = "
        <Step index=\"7\" id=\"6\" name=\"Gehe zu Layout\" enable=\"True\">
            <UUID>BD982069-347C-4B37-A5A4-A1DA05050424</UUID>
            <OwnerID></OwnerID>
            <Options>16384</Options>
            <ParameterValues membercount=\"2\">
                <Parameter type=\"LayoutReferenceContainer\">
                    <LayoutReferenceContainer value=\"3\">
                        <Calculation datatype=\"1\" position=\"5\">
                            <Calculation>
                                <Text><![CDATA[\"Foo\"]]></Text>
                                <ChunkList hash=\"49C67A24DD1D8846365084DF7F1C0809\">
                                    <Chunk type=\"NoRef\">&quot;Foo&quot;</Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </LayoutReferenceContainer>
                </Parameter>
                <Parameter type=\"Animation\">
                    <Animation name=\"Ohne\" value=\"0\"></Animation>
                </Parameter>
            </ParameterValues>
        </Step>
        ";

        let expected_output =
            Some("Gehe zu Layout [ Layoutname: \"Foo\" ; Animation: Ohne ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_sanitize_by_number() {
        let xml_input = "
        <Step index=\"8\" id=\"6\" name=\"Gehe zu Layout\" enable=\"True\">
            <UUID>38D94396-4BAB-45AE-A585-71DB39D0FF68</UUID>
            <OwnerID></OwnerID>
            <Options>16384</Options>
            <ParameterValues membercount=\"2\">
                <Parameter type=\"LayoutReferenceContainer\">
                    <LayoutReferenceContainer value=\"4\">
                        <Calculation datatype=\"1\" position=\"5\">
                            <Calculation>
                                <Text><![CDATA[\"Bar\"]]></Text>
                                <ChunkList hash=\"AB80393374DAB968028CA7E06ACCCEB2\">
                                    <Chunk type=\"NoRef\">&quot;Bar&quot;</Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </LayoutReferenceContainer>
                </Parameter>
                <Parameter type=\"Animation\">
                    <Animation name=\"Ohne\" value=\"0\"></Animation>
                </Parameter>
            </ParameterValues>
        </Step>
        ";

        let expected_output =
            Some("Gehe zu Layout [ Layoutnr.: \"Bar\" ; Animation: Ohne ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }
}
