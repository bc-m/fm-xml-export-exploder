use std::io::{BufRead, BufReader, Read};
use std::mem;
use std::path::{Path, PathBuf};
use std::{fs::File, time::Instant};

use anyhow::{Context, Result, anyhow, bail};
use encoding_rs_io::DecodeReaderBytes;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;

use crate::Skeleton;
use crate::catalog::xml_explode_catalog;
use crate::config::{CatalogType, Flags};
use crate::custom_function_sanitizer::create_sanitized_custom_functions;
use crate::script_sanitizer::create_sanitized_scripts;
use crate::supporting::process_supporting_element;
use crate::utils::attributes::get_attribute;
use crate::utils::xml_utils::{XmlEventType, end_element_to_string, start_element_to_string};
use crate::utils::{
    FolderStructure, VERSION_2_2_3_4, build_out_dir_path, create_dir, delete_output_directory,
    push_line_to_skeleton, version_string_to_number, write_xml_file,
};

pub enum TopLevelSection {
    Structure,
    Metadata,
    DdrInfo,
}

pub enum Action {
    Add,
    Modify,
    Replace,
    Delete,
}

pub enum Qualifier {
    SanitizedScripts,
    SanitizedCustomFunctions,
}

/// Context for XML catalog processing
pub struct ProcessingContext<'a, R: Read + BufRead> {
    pub reader: &'a mut Reader<R>,
    pub path_stack: &'a mut Vec<Vec<u8>>,
    pub root_out_dir: &'a Path,
    pub saxml_version: Option<String>,
    pub saxml_version_num: Option<u64>,
    pub db_name: Option<String>,
    pub top_level_section: Option<TopLevelSection>,
    pub action: Option<Action>,
    pub catalog_type: Option<CatalogType>,
    pub current_out_dir: PathBuf,
    pub skeleton: &'a mut Skeleton,
    pub flags: &'a Flags,
}

/// Process a single XML file and explode it into individual files
pub fn explode_xml(fm_export_file_path: &Path, root_out_dir: &Path, flags: &Flags) -> Result<()> {
    let start = Instant::now();
    let fm_export_file_name = fm_export_file_path.file_name().unwrap().to_str().unwrap();

    // Open XML file
    let file = File::open(fm_export_file_path)
        .with_context(|| format!("Error opening file {}", fm_export_file_path.display()))?;

    // Initialize variables
    let mut depth = 0;
    let mut cf_folder_structure: Option<FolderStructure> = None;
    let mut script_folder_structure: Option<FolderStructure> = None;
    let mut buf = Vec::new();

    // Instantiate processing context which will be passed around to various functions
    let mut context = ProcessingContext {
        reader: &mut Reader::from_reader(BufReader::new(DecodeReaderBytes::new(file))),
        path_stack: &mut Vec::new(),
        root_out_dir,
        saxml_version: None,
        saxml_version_num: None,
        db_name: None,
        top_level_section: None,
        action: None,
        catalog_type: None,
        current_out_dir: PathBuf::new(),
        skeleton: &mut Skeleton::default(),
        flags,
    };

    // Iterate over XML events
    loop {
        match context.reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("Error in {}: {}", fm_export_file_path.display(), e);
                break;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(start_tag)) => {
                context.path_stack.push(start_tag.name().as_ref().to_vec());
                push_start_to_skeleton(
                    &start_tag,
                    context.skeleton,
                    context.path_stack,
                    context.flags,
                );
                match depth {
                    0 => {
                        process_root_element(&mut context, &start_tag)?;
                        create_dir(context.root_out_dir);
                        delete_output_directory(&context)?;
                    }
                    1 => {
                        let was_xml_element_consumed =
                            process_top_level_section(&mut context, &start_tag)?;
                        if was_xml_element_consumed {
                            continue;
                        }
                    }
                    2 => {
                        context.action = match start_tag.name().as_ref() {
                            b"AddAction" => Some(Action::Add),
                            b"ModifyAction" => Some(Action::Modify),
                            b"ReplaceAction" => Some(Action::Replace),
                            b"DeleteAction" => Some(Action::Delete),
                            _ => None,
                        };
                    }
                    3 => {
                        let is_supported_catalog = process_catalog_elements(
                            &mut context,
                            &start_tag,
                            &mut cf_folder_structure,
                            &mut script_folder_structure,
                        )?;
                        if is_supported_catalog {
                            continue;
                        }
                    }
                    _ => {}
                }
                depth += 1;
            }
            Ok(Event::End(e)) => {
                context.path_stack.pop();
                push_end_to_skeleton(&e, context.skeleton, context.path_stack, context.flags);
                depth -= 1;
            }
            _ => {}
        }
        buf.clear()
    }

    // Write skeleton file if in lossless mode
    if flags.lossless {
        write_skeleton_file(&context)?;
    }

    println!(
        "→ {} finished in {} ms.",
        fm_export_file_name,
        start.elapsed().as_millis()
    );

    Ok(())
}

fn process_root_element<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    e: &BytesStart,
) -> Result<()> {
    match e.name().as_ref() {
        b"FMDynamicTemplate" | b"FMSaveAsXML" => {}
        _ => bail!("Unsupported XML-format"),
    }

    let file_attr = get_attribute(e, "File")
        .ok_or_else(|| anyhow!("Missing 'File' attribute on root element"))?;
    context.db_name = Some(
        file_attr
            .strip_suffix(".fmp12")
            .unwrap_or(&file_attr)
            .to_string(),
    );
    let saxml_version = get_attribute(e, "version")
        .ok_or_else(|| anyhow!("Missing 'version' attribute on root element"))?;
    context.saxml_version_num = Some(version_string_to_number(&saxml_version));
    context.saxml_version = Some(saxml_version);
    Ok(())
}

/// Process supporting elements (Metadata, DDR_INFO) that are not part of the main structure
fn process_top_level_section<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    start_tag: &BytesStart,
) -> Result<bool> {
    let (section, out_file_name) = match start_tag.name().as_ref() {
        b"Structure" => {
            context.top_level_section = Some(TopLevelSection::Structure);
            return Ok(false);
        }
        b"Metadata" => (TopLevelSection::Metadata, "metadata"),
        b"DDR_INFO" => (TopLevelSection::DdrInfo, "ddr_info"),
        _ => {
            context.top_level_section = None;
            return Ok(false);
        }
    };

    context.top_level_section = Some(section);
    process_supporting_element(context, start_tag, out_file_name)?;
    Ok(true)
}

fn process_catalog_elements<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    start_tag: &BytesStart,
    cf_folder_structure: &mut Option<FolderStructure>,
    script_folder_structure: &mut Option<FolderStructure>,
) -> Result<bool> {
    // Detect catalog type
    let Some(catalog_type) = CatalogType::from_bytes(start_tag.name().as_ref()) else {
        return Ok(false);
    };
    context.catalog_type = Some(catalog_type);

    let ver = context.saxml_version_num.unwrap_or(0);

    if catalog_type == CatalogType::OptionsForValueLists && ver >= VERSION_2_2_3_4 {
        return Ok(true);
    }
    // Handle special cases that need folder structures
    let folder_structure = match catalog_type {
        CatalogType::CalcsForCustomFunctions => cf_folder_structure.as_ref(),
        CatalogType::StepsForScripts => script_folder_structure.as_ref(),
        _ => None,
    };

    let xml_out_dir_path = build_out_dir_path(context, None)?;
    let base_dir_path = mem::replace(&mut context.current_out_dir, xml_out_dir_path.clone());

    let folder_structure_result = xml_explode_catalog(context, folder_structure)?;

    context.current_out_dir = base_dir_path;

    // Update folder structures for catalogs that return them
    if let Some(folder_structure) = folder_structure_result {
        match catalog_type {
            CatalogType::CustomFunctions => *cf_folder_structure = Some(folder_structure),
            CatalogType::Script => *script_folder_structure = Some(folder_structure),
            _ => {}
        }
    }

    // Post-processing: create sanitized (human-readable) versions
    match catalog_type {
        CatalogType::StepsForScripts => {
            let dir = build_out_dir_path(context, Some(Qualifier::SanitizedScripts))?;
            create_sanitized_scripts(&xml_out_dir_path, &dir, context.flags);
        }
        CatalogType::CalcsForCustomFunctions => {
            let dir = build_out_dir_path(context, Some(Qualifier::SanitizedCustomFunctions))?;
            create_sanitized_custom_functions(&xml_out_dir_path, &dir);
        }
        CatalogType::CustomFunctions if ver >= VERSION_2_2_3_4 => {
            let dir = build_out_dir_path(context, Some(Qualifier::SanitizedCustomFunctions))?;
            create_sanitized_custom_functions(&xml_out_dir_path, &dir);
        }
        _ => {}
    }
    Ok(true) // is_supported_catalog
}

fn write_skeleton_file<R: Read + BufRead>(context: &ProcessingContext<'_, R>) -> Result<()> {
    let skeleton_dir_path = build_out_dir_path(context, None)?;
    let skeleton_file_path = skeleton_dir_path.join("skeleton.xml");
    write_xml_file(
        &skeleton_file_path,
        &context.skeleton.content,
        0,
        context.flags,
    );
    Ok(())
}

fn push_start_to_skeleton(
    e: &BytesStart,
    skeleton: &mut Skeleton,
    path_stack: &[Vec<u8>],
    flags: &Flags,
) {
    if !flags.lossless {
        return;
    }

    // After pushing, path_stack.len() is depth+1 (1-indexed).
    // We want skeleton entries for depths 1-3 unconditionally,
    // and depths 4-5 only under an AddAction/ModifyAction ancestor.
    if should_add_to_skeleton(path_stack.len(), 1, path_stack) {
        push_line_to_skeleton(
            skeleton,
            path_stack.len(),
            1,
            &start_element_to_string(e, flags),
            true,
            XmlEventType::Start,
        );
    }
}

fn push_end_to_skeleton(
    e: &BytesEnd,
    skeleton: &mut Skeleton,
    path_stack: &[Vec<u8>],
    flags: &Flags,
) {
    if !flags.lossless {
        return;
    }

    // After popping, path_stack.len() is depth (0-indexed from root).
    // We want skeleton entries for depths 0-2 unconditionally,
    // and depths 3-4 only under an AddAction/ModifyAction ancestor.
    if should_add_to_skeleton(path_stack.len(), 0, path_stack) {
        push_line_to_skeleton(
            skeleton,
            path_stack.len(),
            1,
            &end_element_to_string(e),
            false,
            XmlEventType::End,
        );
    }
}

/// Determine whether an element at the given depth should be added to the skeleton.
/// `base` adjusts the depth thresholds: 1 for start events (1-indexed after push),
/// 0 for end events (0-indexed after pop).
fn should_add_to_skeleton(depth: usize, base: usize, path_stack: &[Vec<u8>]) -> bool {
    let max_unconditional = 2 + base; // 3 for start, 2 for end
    let max_conditional = 4 + base; // 5 for start, 4 for end
    depth <= max_unconditional || (depth <= max_conditional && is_action_ancestor(path_stack))
}

/// Check whether the path_stack contains an AddAction or ModifyAction ancestor
/// at position index 2 (the action level in the XML tree).
fn is_action_ancestor(path_stack: &[Vec<u8>]) -> bool {
    path_stack
        .get(2)
        .is_some_and(|v| matches!(v.as_slice(), b"AddAction" | b"ModifyAction"))
}
