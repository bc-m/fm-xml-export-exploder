use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};

use crate::utils::attributes::get_attributes;
use crate::utils::file_utils::{escape_filename, join_scope_id_and_name};
use crate::utils::xml_utils::{
    XmlEventType, end_element_to_string, extract_values_from_xml_paths, format_end_tag,
    push_rest_of_element_to_skeleton, skip_rest_of_element, start_element_to_string,
};
use crate::utils::{
    FolderStructure, build_out_dir_path, create_dir, move_to_subfolder, push_line_to_skeleton,
    rename_file_if_necessary, write_rest_of_element_to_file,
};
use crate::xml_processor::ProcessingContext;

/// Parse and explode a generic catalog, handling both wrapped and unwrapped formats
pub fn xml_explode_catalog<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    folder_structure: Option<&FolderStructure>,
) -> Result<Option<FolderStructure>> {
    let Some(catalog_type) = context.catalog_type else {
        bail!("Catalog type not specified");
    };
    let catalog_config = catalog_type.get_config();
    let catalog_item_name = catalog_config.catalog_item_name;
    let wrapped_in_object_list = catalog_config.wrapped_in_object_list;
    let uses_folders = catalog_config.uses_folders;
    let id_path = &catalog_config.id_path;

    let out_dir_path_base = build_out_dir_path(context, None)?;
    create_dir(&out_dir_path_base);

    let mut buf = Vec::new(); // buffer for reading xml events

    // Adjust depth based on whether items are wrapped in ObjectList
    let base_depth = context.path_stack.len(); // depth of the catalog start tag, e.g., 4 for BaseDirectoryCatalog, if the path is FMSaveAsXML/Structure/AddAction/BaseDirectoryCatalog
    let mut rel_depth = 1; // tracks depth as we traverse the xml; depth 1 is the catalog start tag, e.g., <BaseDirectoryCatalog>
    let target_depth = if wrapped_in_object_list { 3 } else { 2 }; // depth of repeating catalog child item, e.g., <BaseDirectory>

    // Folder tracking variables
    let mut current_path: Vec<String> = Vec::new();
    let mut current_id = String::new();
    let mut current_name = String::new();

    // We'll build a folder structure for custom functions, layouts, scripts, etc.
    let mut folder_structure_result = if uses_folders {
        Some(FolderStructure::default())
    } else {
        None
    };

    loop {
        match context.reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("Error {e}");
                break;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                rel_depth += 1;
                let start_tag = start_element_to_string(&e, context.flags);

                // Add repeating catalog child items (e.g. <BaseDirectory>) and their siblings, if any, to the skeleton (e.g. <UUID>)
                // If repeating catalog child items are wrapped in ObjectList (e.g. <Account>),
                // the siblings will be siblings of ObjectList instead of the catalog item
                add_start_tag_to_skeleton(
                    context,
                    &start_tag,
                    base_depth,
                    rel_depth,
                    wrapped_in_object_list,
                );

                // Is the current element an ancillary element?
                // Ancillary elements are elements that are not catalog items or <ObjectList>, e.g., <UUID> or <TagList>
                let expected_element = if wrapped_in_object_list {
                    b"ObjectList".as_ref()
                } else {
                    catalog_item_name
                };
                let is_ancillary_element = rel_depth == 2 && e.name().as_ref() != expected_element;

                // Handle ancillary elements like <UUID> and <TagList>
                if is_ancillary_element {
                    handle_ancillary_element(context, base_depth, rel_depth);
                    rel_depth -= 1;
                }

                // Skip rest of loop if it's a non-catalog item (e.g. <ObjectList> at depth 2)
                if rel_depth != target_depth || e.name().as_ref() != catalog_item_name {
                    continue;
                }

                // Handle catalog items (e.g. <BaseDirectory>)
                // Parse attributes needed for folder tracking
                if uses_folders {
                    let attrs = parse_folder_attributes(&e);
                    current_id = attrs.id;
                    current_name = attrs.name;

                    if attrs.is_folder {
                        current_path.push(join_scope_id_and_name(
                            &current_id,
                            &escape_filename(&current_name),
                        ));
                    }
                    if attrs.is_marker {
                        current_path.pop();
                    }

                    if (attrs.is_folder || attrs.is_marker || attrs.is_separator)
                        && !context.flags.lossless
                    {
                        skip_rest_of_element(context.reader);
                        rel_depth -= 1;
                        continue;
                    }
                } else {
                    // Reset folder tracking variables for each catalog item
                    current_id.clear();
                    current_name.clear();
                }

                // Adjust indentation level based on wrapping
                let indentation_level = if wrapped_in_object_list { 5 } else { 4 };
                let file_path = write_rest_of_element_to_file(
                    context,
                    &e,
                    indentation_level,
                    base_depth + rel_depth - 1,
                    id_path,
                );
                rename_file_if_necessary(&file_path, context.path_stack, catalog_item_name);

                // Move to subfolder if necessary
                if let Some(subfolder) = determine_subfolder_path(
                    &out_dir_path_base,
                    folder_structure,
                    &file_path,
                    id_path,
                    uses_folders,
                    &current_path,
                ) {
                    let _ = move_to_subfolder(&file_path, &subfolder);
                }

                update_folder_structure(&mut folder_structure_result, &current_id, &current_path);

                // The element will be consumed by now, so we can't rely on catching the end tag in the Ok(Event::End) arm below
                // Instead, write it out manually here
                if context.flags.lossless {
                    let end_tag = format_end_tag(e.name().as_ref());
                    push_line_to_skeleton(
                        context.skeleton,
                        base_depth,
                        rel_depth - 1,
                        &end_tag,
                        false,
                        XmlEventType::End,
                    );
                }

                rel_depth -= 1;
            }
            Ok(Event::End(e)) => {
                let end_tag = end_element_to_string(&e);
                rel_depth -= 1;

                // Add to skeleton only if it's a direct catalog child or the catalog end tag
                if context.flags.lossless && rel_depth <= 1 {
                    push_line_to_skeleton(
                        context.skeleton,
                        base_depth,
                        rel_depth,
                        &end_tag,
                        false,
                        XmlEventType::End,
                    );
                }

                if rel_depth == 0 {
                    context.path_stack.pop();
                    break;
                }
            }
            _ => {}
        }

        buf.clear()
    }

    Ok(folder_structure_result)
}

/// Add start tag to skeleton if conditions are met
fn add_start_tag_to_skeleton<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    start_tag: &str,
    base_depth: usize,
    rel_depth: usize,
    wrapped_in_object_list: bool,
) {
    let should_add = rel_depth == 2 || (wrapped_in_object_list && rel_depth == 3);
    if !context.flags.lossless || !should_add {
        return;
    }

    // When should_add is true, the element is always a child of the catalog:
    // depth 2 is always a child, and depth 3 only passes when wrapped_in_object_list
    push_line_to_skeleton(
        context.skeleton,
        base_depth,
        rel_depth,
        start_tag,
        true,
        XmlEventType::Start,
    );
}

#[derive(Default)]
struct FolderAttributes {
    id: String,
    name: String,
    is_folder: bool,
    is_marker: bool,
    is_separator: bool,
}

/// Parse folder-related attributes from a catalog item
fn parse_folder_attributes(e: &BytesStart) -> FolderAttributes {
    let mut attrs = FolderAttributes::default();

    for attr in get_attributes(e) {
        match attr.0.as_str() {
            "id" => attrs.id = attr.1,
            "name" => attrs.name = unescape(&attr.1).unwrap().into_owned(),
            "isFolder" => match attr.1.as_str() {
                "True" => attrs.is_folder = true,
                "Marker" => attrs.is_marker = true,
                _ => {}
            },
            "isSeparatorItem" => attrs.is_separator = attr.1 == "True",
            _ => {}
        }
    }

    attrs
}

/// Determine the subfolder path for a catalog item.
/// Uses `current_path` for catalogs that track their own folder hierarchy (scripts, custom functions, layouts),
/// or looks up the path via `folder_structure` for dependent catalogs (steps, calcs).
/// Returns `Some(subfolder)` only when the item should be moved, i.e., the resolved
/// subfolder differs from `out_dir_path_base`.
fn determine_subfolder_path(
    out_dir_path_base: &Path,
    folder_structure: Option<&FolderStructure>,
    file_path: &Path,
    id_path: &str,
    uses_folders: bool,
    current_path: &[String],
) -> Option<PathBuf> {
    let subfolder = match folder_structure {
        Some(fs) => {
            // For dependent catalogs that use a previously-built folder structure
            if id_path.is_empty() {
                return None;
            }
            let results = extract_values_from_xml_paths(file_path, &[id_path]).ok()?;
            let id = results.first()?.as_ref()?;
            let path = fs.get_path_for_id(id);
            if path.is_empty() {
                return None;
            }
            out_dir_path_base.join(path.join("/"))
        }
        None => {
            // For catalogs with their own folder tracking
            if !uses_folders || current_path.is_empty() {
                return None;
            }
            out_dir_path_base.join(current_path.join("/"))
        }
    };

    (subfolder != out_dir_path_base).then_some(subfolder)
}

/// Handle ancillary elements like <UUID> and <TagList>
fn handle_ancillary_element<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    base_depth: usize,
    rel_depth: usize,
) {
    if context.flags.lossless {
        push_rest_of_element_to_skeleton(
            context.reader,
            context.skeleton,
            base_depth + rel_depth - 1,
            context.flags,
        );
    } else {
        skip_rest_of_element(context.reader);
    }
}

/// Update folder structure with current item
fn update_folder_structure(
    folder_structure_result: &mut Option<FolderStructure>,
    current_id: &str,
    current_path: &[String],
) {
    if let Some(folder_structure) = folder_structure_result {
        folder_structure
            .item_paths
            .insert(current_id.to_string(), current_path.to_vec());
    }
}
