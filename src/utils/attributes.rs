use std::borrow::Cow;

use quick_xml::events::BytesStart;
use quick_xml::name::QName;

pub fn parse_unescaped_attribute(e: &BytesStart, attribute: &str) -> Option<String> {
    get_attribute(e, attribute).map(|text| quick_xml::escape::unescape(&text).unwrap().to_string())
}

pub fn key_to_string(key: QName) -> String {
    String::from_utf8_lossy(key.as_ref()).to_string()
}

pub fn value_to_string(value: Cow<[u8]>) -> String {
    String::from_utf8_lossy(value.as_ref()).to_string()
}

pub fn get_attributes(e: &BytesStart) -> Option<Vec<(String, String)>> {
    let mut attributes = Vec::new();

    for attr in e.attributes() {
        let attr = match attr {
            Ok(attribute) => attribute,
            Err(_) => continue,
        };

        let key = key_to_string(attr.key);
        let value = value_to_string(attr.value);
        attributes.push((key, value));
    }

    Some(attributes)
}

pub fn get_attribute(element: &BytesStart, attribute_name: &str) -> Option<String> {
    for attr in get_attributes(element).unwrap() {
        if attr.0 == attribute_name {
            return Some(attr.1);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

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
