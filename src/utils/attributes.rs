use quick_xml::events::BytesStart;

/// Resolves XML entities, e.g. `&apos;` becomes `'`. Malformed entities are kept as is.
pub fn unescape_value(text: &str) -> String {
    quick_xml::escape::unescape(text).map_or_else(|_| text.to_string(), |v| v.into_owned())
}

pub fn get_attributes(e: &BytesStart) -> Vec<(String, String)> {
    e.attributes()
        .filter_map(|attr| attr.ok())
        .map(|attr| (attr.key.as_ref().to_owned(), attr.value.into_owned()))
        .collect()
}

pub fn get_attribute(element: &BytesStart, attribute_name: &str) -> Option<String> {
    element
        .attributes()
        .filter_map(|attr| attr.ok())
        .find(|attr| attr.key.as_ref() == attribute_name)
        .map(|attr| attr.value.into_owned())
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use crate::utils;

    #[test]
    fn test_unescape_value() {
        assert_eq!(
            utils::attributes::unescape_value("object&apos;s &lt;Unknown&gt;"),
            "object's <Unknown>"
        );
        // Malformed entities are kept instead of panicking
        assert_eq!(utils::attributes::unescape_value("A & B"), "A & B");
    }

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
