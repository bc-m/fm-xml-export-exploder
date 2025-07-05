use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};

use anyhow::Error;
use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};

use crate::utils::attributes::get_attributes;
use crate::utils::file_utils::join_scope_id_and_name;
use crate::utils::xml_utils::{
    end_element_to_string, end_element_to_string_from_start_element, extract_values_from_xml_paths,
    push_rest_of_element_to_skeleton, skip_rest_of_element, start_element_to_string, XmlEventType,
};
use crate::utils::{
    build_out_dir_path, create_dir, move_to_subfolder, write_rest_of_element_to_file,
    FolderStructure,
};
use crate::utils::{push_line_to_skeleton, rename_file_if_necessary};
use crate::xml_processor::ProcessingContext;

/// Parse and explode a generic catalog, handling both wrapped and unwrapped formats
pub fn xml_explode_catalog<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    start_tag: &BytesStart,
    folder_structure: Option<&FolderStructure>,
    // catalog_config: &CatalogConfig,
) -> Result<Option<FolderStructure>, Error> {
    let catalog_type = match context.catalog_type {
        Some(catalog_type) => catalog_type,
        None => return Err(anyhow::anyhow!("❌ Catalog type not specified")),
    };
    let catalog_config = catalog_type.get_config();
    let catalog_item_name = catalog_config.catalog_item_name.clone();
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
        Some(FolderStructure::new())
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
                let is_ancillary_element = rel_depth == 2
                    && (
                        // Siblings of ObjectList wrapper at depth 2
                        (wrapped_in_object_list && e.name().as_ref() != b"ObjectList")
                        // When not wrapped in <ObjectList>, siblings of catalog items
                        || (!wrapped_in_object_list && e.name().as_ref() != catalog_item_name)
                    );

                // Handle ancillary elements like <UUID> and <TagList>
                if is_ancillary_element {
                    handle_ancillary_element(context, &e, base_depth, rel_depth);
                    rel_depth -= 1;
                }

                // Skip rest of loop if it's a non-catalog item (e.g. <ObjectList> at depth 2)
                if rel_depth != target_depth || e.name().as_ref() != catalog_item_name {
                    continue;
                }

                // Handle catalog items (e.g. <BaseDirectory>)
                // Parse attributes needed for folder tracking
                if uses_folders {
                    let (id, name, is_folder, is_marker) = parse_folder_attributes(&e);
                    current_id = id;
                    current_name = name;

                    if is_folder {
                        current_path.push(join_scope_id_and_name(&current_id, &current_name));
                    }
                    if is_marker {
                        current_path.pop();
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
                rename_file_if_necessary(&file_path, context.path_stack, &catalog_item_name);

                // Move to subfolder if necessary
                let subfolder_dir_path = determine_subfolder_path(
                    &out_dir_path_base,
                    folder_structure,
                    &file_path,
                    id_path,
                    uses_folders,
                    &current_path,
                );
                if let Some(subfolder_dir_path) = subfolder_dir_path {
                    if subfolder_dir_path != out_dir_path_base
                        && !subfolder_dir_path.to_string_lossy().is_empty()
                    {
                        let _ = move_to_subfolder(&file_path, &subfolder_dir_path);
                    }
                }

                update_folder_structure(&mut folder_structure_result, &current_id, &current_path);

                // The element will be consumed by now, so we can't rely on catching the end tag in the Ok(Event::End) arm below
                // Instead, write it out manually here
                if context.flags.lossless {
                    let end_tag = end_element_to_string_from_start_element(&e);
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
    if !context.flags.lossless {
        return;
    }

    let should_add_to_skeleton = rel_depth == 2 || (wrapped_in_object_list && rel_depth == 3);
    if !should_add_to_skeleton {
        return;
    }

    push_line_to_skeleton(
        context.skeleton,
        base_depth,
        rel_depth,
        start_tag,
        if wrapped_in_object_list {
            rel_depth >= 2
        } else {
            rel_depth == 2
        },
        XmlEventType::Start,
    );
}

/// Parse folder-related attributes from a catalog item
fn parse_folder_attributes(e: &BytesStart) -> (String, String, bool, bool) {
    let mut current_id = String::new();
    let mut current_name = String::new();
    let mut is_folder = false;
    let mut is_marker = false;

    for attr in get_attributes(e).unwrap() {
        match attr.0.as_str() {
            "id" => current_id = attr.1.to_string(),
            "name" => current_name = unescape(attr.1.as_str()).unwrap().to_string(),
            "isFolder" => match attr.1.as_str() {
                "True" => is_folder = true,
                "Marker" => is_marker = true,
                _ => {}
            },
            _ => {}
        }
    }

    (current_id, current_name, is_folder, is_marker)
}

/// Determine the subfolder path for a catalog item
fn determine_subfolder_path(
    out_dir_path_base: &Path,
    folder_structure: Option<&FolderStructure>,
    file_path: &Path,
    id_path: &str,
    uses_folders: bool,
    current_path: &[String],
) -> Option<PathBuf> {
    let folder_structure = match folder_structure {
        Some(folder_structure) => folder_structure,
        None => {
            return if uses_folders && !current_path.is_empty() {
                // Track folders using current catalog item
                Some(out_dir_path_base.join(current_path.join("/")))
            } else {
                None
            };
        }
    };

    // If a folder structure was provided, use that
    if id_path.is_empty() {
        return None;
    }

    let paths = vec![id_path];
    let results = match extract_values_from_xml_paths(file_path, &paths) {
        Ok(results) => results,
        Err(_) => return None,
    };

    let id = match results.first() {
        Some(Some(id)) => id,
        _ => return None,
    };

    let function_path = folder_structure.get_path_for_id(id);
    if function_path.is_empty() {
        return None;
    }

    Some(out_dir_path_base.join(function_path.join("/")))
}

/// Handle ancillary elements like <UUID> and <TagList>
fn handle_ancillary_element<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    e: &BytesStart,
    base_depth: usize,
    rel_depth: usize,
) {
    if context.flags.lossless {
        push_rest_of_element_to_skeleton(
            context.reader,
            e,
            context.skeleton,
            base_depth + rel_depth - 1,
            context.flags,
        );
    } else {
        skip_rest_of_element(context.reader, e);
    }
}

/// Update folder structure with current item
fn update_folder_structure(
    folder_structure_result: &mut Option<FolderStructure>,
    current_id: &str,
    current_path: &[String],
) {
    if let Some(ref mut folder_structure) = folder_structure_result {
        folder_structure
            .item_paths
            .insert(current_id.to_string(), current_path.to_vec());
    }
}
