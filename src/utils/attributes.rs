use std::borrow::Cow;

use quick_xml::events::BytesStart;
use quick_xml::name::QName;

pub fn parse_unescaped_attribute(e: &BytesStart, attribute: &str) -> Option<String> {
    get_attribute(e, attribute).map(|text| quick_xml::escape::unescape(&text).unwrap().into_owned())
}

pub fn key_to_string(key: QName) -> String {
    String::from_utf8_lossy(key.as_ref()).to_string()
}

pub fn value_to_string(value: Cow<[u8]>) -> String {
    String::from_utf8_lossy(value.as_ref()).to_string()
}

pub fn get_attributes(e: &BytesStart) -> Vec<(String, String)> {
    e.attributes()
        .filter_map(|attr| attr.ok())
        .map(|attr| (key_to_string(attr.key), value_to_string(attr.value)))
        .collect()
}

pub fn get_attribute(element: &BytesStart, attribute_name: &str) -> Option<String> {
    element
        .attributes()
        .filter_map(|attr| attr.ok())
        .find(|attr| attr.key.as_ref() == attribute_name.as_bytes())
        .map(|attr| value_to_string(attr.value))
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use crate::utils;

    #[test]
    fn test_get_name_attribute() {
        let xml_tag = "<Name value=\"$Serverscript\">";
        let mut reader = Reader::from_str(xml_tag);

        if let Ok(Event::Start(ref e)) = reader.read_event() {
            assert_eq!(
                utils::attributes::get_attribute(e, "value").unwrap(),
                "$Serverscript"
            );
        }
    }
}
