use quick_xml::events::BytesStart;

pub fn parse_unescaped_attribute(e: &BytesStart, attribute: &str) -> Option<String> {
    get_attribute(e, attribute).map(|text| quick_xml::escape::unescape(&text).unwrap().into_owned())
}

pub fn get_attributes(e: &BytesStart) -> Vec<(String, String)> {
    e.attributes()
        .filter_map(|attr| attr.ok())
        .map(|attr| {
            (
                String::from_utf8_lossy(attr.key.as_ref()).into_owned(),
                String::from_utf8_lossy(&attr.value).into_owned(),
            )
        })
        .collect()
}

pub fn get_attribute(element: &BytesStart, attribute_name: &str) -> Option<String> {
    element
        .attributes()
        .filter_map(|attr| attr.ok())
        .find(|attr| attr.key.as_ref() == attribute_name.as_bytes())
        .map(|attr| String::from_utf8_lossy(&attr.value).into_owned())
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
