use std::io::{BufRead, Read};

use quick_xml::events::{BytesStart, Event};

use crate::utils::attributes::get_attributes;
use crate::utils::file_utils::{escape_filename, join_scope_id_and_name};
use crate::utils::skeleton::push_line_to_skeleton;
use crate::utils::xml_utils::{
    XmlEventType, element_to_string, end_element_to_string, general_ref_to_string,
    start_element_to_string, text_element_to_string, unescape_xml_entities,
};
use crate::xml_processor::ProcessingContext;

#[derive(Debug, Default)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub tag_name: String,
    pub element_with_id: String,
    pub content: String,
}

impl Entity {
    /// The default file name, built from the id and name of the catalog item
    pub fn file_name(&self) -> String {
        format!(
            "{}.xml",
            escape_filename(&join_scope_id_and_name(&self.id, &self.name))
        )
    }

    fn parse_xml_attributes(&mut self, e: &BytesStart) {
        for attr in get_attributes(e) {
            match attr.0.as_str() {
                "id" => self.id = attr.1,
                "name" if self.name.is_empty() => self.name = unescape_xml_entities(attr.1),
                "Display" => self.name = unescape_xml_entities(attr.1),
                _ => {}
            }
        }
    }

    pub fn read_xml_element<R: Read + BufRead>(
        &mut self,
        context: &mut ProcessingContext<'_, R>,
        start_tag: &BytesStart,
        id_path: &str,
    ) {
        self.parse_xml_attributes(start_tag);
        self.tag_name = start_tag.name().as_ref().to_owned();

        if !self.id.is_empty() {
            let element_string = element_to_string(context, start_tag);
            self.content.push_str(&element_string);
            if !id_path.is_empty() {
                let parent_element = id_path.rsplit('/').nth(1).unwrap_or("unknown");
                if parent_element == self.tag_name {
                    self.element_with_id = element_string;
                }
            }
            return;
        }

        self.content
            .push_str(&start_element_to_string(start_tag, context.flags));
        let mut buf = Vec::new();
        loop {
            match context.reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    self.read_xml_element(context, &e, id_path);
                }
                Ok(Event::Text(e)) => {
                    self.content.push_str(&text_element_to_string(&e, false));
                }
                Ok(Event::GeneralRef(e)) => {
                    self.content.push_str(&general_ref_to_string(&e, true));
                }
                Ok(Event::End(e)) => {
                    self.content.push_str(&end_element_to_string(&e));
                    break;
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
    }
}

/// Reads a catalog item into memory; an element with an id is also added to the skeleton.
pub fn read_rest_of_element<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    start_tag: &BytesStart,
    base_depth: usize,
    id_path: &str,
) -> Entity {
    let mut entity = Entity::default();
    entity.read_xml_element(context, start_tag, id_path);
    if !entity.element_with_id.is_empty() {
        push_line_to_skeleton(
            context.skeleton,
            base_depth,
            1,
            &entity.element_with_id,
            false,
            XmlEventType::Other,
        );
    }
    entity
}
