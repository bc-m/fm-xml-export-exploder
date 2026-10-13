use quick_xml::Reader;
use quick_xml::events::Event;

use crate::script_steps::parameters::boolean::Boolean;
use crate::script_steps::parameters::calculation::Calculation;
use crate::script_steps::parameters::target::Target;
use crate::utils::attributes::get_attribute;

const VERIFY_SSL_CERTIFICATES: &str = "268435456";
const SELECT: &str = "4096";
const WITH_DIALOG: &str = "128";

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut select: Option<String> = None;
    let mut with_dialog: Option<String> = None;
    let mut target: Option<String> = None;
    let mut url: Option<String> = None;
    // "Verify SSL Certificates" is shown by name only when enabled, mirroring FileMaker.
    let mut verify_ssl: Option<String> = None;
    let mut curl_options: Option<String> = None;

    let mut reader = Reader::from_str(step);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                "Step" => {
                    name = get_attribute(&e, "name").unwrap_or_default();
                }
                "Boolean" => {
                    let id = get_attribute(&e, "id").unwrap_or_default();
                    let label = get_attribute(&e, "type").unwrap_or_default();
                    let is_on = get_attribute(&e, "value").as_deref() == Some("True");
                    match id.as_str() {
                        VERIFY_SSL_CERTIFICATES if is_on => verify_ssl = Some(label),
                        SELECT if is_on => select = Some(label),
                        WITH_DIALOG => {
                            with_dialog = Some(format!("{label}: {}", Boolean::on_off(is_on)))
                        }
                        _ => {}
                    }
                }
                "Parameter" => match get_attribute(&e, "type").as_deref() {
                    Some("Target") => target = Target::from_xml(&mut reader, &e).display(),
                    Some("URL") => {
                        url = Calculation::from_xml(&mut reader, &e)
                            .display()
                            .map(|c| format!("URL: {c}"))
                    }
                    Some("Calculation") => {
                        curl_options = Calculation::from_xml(&mut reader, &e)
                            .display()
                            .map(|c| format!("cURL options: {c}"))
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
        buf.clear();
    }

    if name.is_empty() {
        return None;
    }

    let parts: Vec<String> = [
        select,
        with_dialog,
        target.map(|t| format!("Target: {t}")),
        url,
        verify_ssl,
        curl_options,
    ]
    .into_iter()
    .flatten()
    .collect();

    if parts.is_empty() {
        Some(name)
    } else {
        Some(format!("{name} [ {} ]", parts.join(" ; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_url() {
        let xml = r#"
            <Step index="11" id="160" name="Insert from URL" enable="True">
                <ParameterValues membercount="4">
                    <Parameter type="Boolean">
                        <Boolean type="Verify SSL Certificates" id="268435456" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Select" id="4096" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="With dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="URL">
                        <URL autoEncode="True"></URL>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Insert from URL [ Select ; With dialog: OFF ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn full_with_verify_ssl_and_curl() {
        let xml = r#"
            <Step index="0" id="160" name="Insert from URL" enable="True">
                <ParameterValues membercount="6">
                    <Parameter type="Boolean">
                        <Boolean type="Verify SSL Certificates" id="268435456" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Select" id="4096" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="With dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Target">
                        <Variable value="$response">
                            <repetition value="1"></repetition>
                        </Variable>
                    </Parameter>
                    <Parameter type="URL">
                        <URL autoEncode="False">
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA[$url]]></Text>
                                </Calculation>
                            </Calculation>
                        </URL>
                    </Parameter>
                    <Parameter type="Calculation">
                        <Calculation datatype="1" position="1">
                            <Calculation>
                                <Text><![CDATA[$curl]]></Text>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Insert from URL [ Select ; With dialog: OFF ; Target: $response ; URL: $url ; Verify SSL Certificates ; cURL options: $curl ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn url_without_target_or_curl() {
        let xml = r#"
            <Step index="0" id="160" name="Insert from URL" enable="True">
                <ParameterValues membercount="4">
                    <Parameter type="Boolean">
                        <Boolean type="Verify SSL Certificates" id="268435456" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Select" id="4096" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="With dialog" id="128" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="URL">
                        <URL autoEncode="True">
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA[$url]]></Text>
                                </Calculation>
                            </Calculation>
                        </URL>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Insert from URL [ With dialog: ON ; URL: $url ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
