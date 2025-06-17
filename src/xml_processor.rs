use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::{fs::File, time::Instant};

use anyhow::{bail, Context, Error, Result};
use encoding_rs_io::DecodeReaderBytes;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;

use crate::catalog::xml_explode_catalog;
use crate::config::{CatalogType, Flags};
use crate::script_sanitizer::create_sanitized_scripts;
use crate::supporting::process_supporting_element;
use crate::utils::attributes::get_attribute;
use crate::utils::xml_utils::{end_element_to_string, start_element_to_string, XmlEventType};
use crate::utils::{add_fm_name_to_base_path, build_out_dir_path, write_xml_file, FolderStructure};
use crate::utils::{push_line_to_skeleton, version_string_to_number};
use crate::Skeleton;

/// Context for XML catalog processing
pub struct ProcessingContext<'a, R: Read + BufRead> {
    pub reader: &'a mut Reader<R>,
    pub path_stack: &'a mut Vec<Vec<u8>>,
    pub current_out_dir: PathBuf,
    pub skeleton: &'a mut Skeleton,
    pub flags: &'a Flags,
}

/// Process a single XML file and explode it into individual files
pub fn explode_xml(
    fm_export_file_path: &PathBuf,
    root_out_dir: &Path,
    flags: &Flags,
) -> Result<(), Error> {
    let start = Instant::now();
    let fm_export_file_name = fm_export_file_path.file_name().unwrap().to_str().unwrap();

    // Open XML file
    let file = File::open(fm_export_file_path)
        .with_context(|| format!("Error opening file {}", fm_export_file_path.display(),))?;

    // Initialize variables
    let mut fm_file_name = String::new();
    let mut saxml_version = String::new();
    let mut depth = 0;
    let mut cf_folder_structure: Option<FolderStructure> = None;
    let mut script_folder_structure: Option<FolderStructure> = None;
    let mut buf = Vec::new();

    // Instantiate processing context which will be passed around to various functions
    let mut context = ProcessingContext {
        reader: &mut Reader::from_reader(BufReader::new(DecodeReaderBytes::new(file))),
        path_stack: &mut Vec::new(),
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
                // context.start_tag = Some(&e);
                // if depth <= 4 {
                //     println!("📟 {}:{} {}", depth, "  ".repeat(depth), start_element_to_string(&e, flags));
                // }
                context.path_stack.push(start_tag.name().as_ref().to_vec());
                push_start_to_skeleton(
                    &start_tag,
                    context.skeleton,
                    context.path_stack,
                    context.flags,
                );
                match depth {
                    0 => {
                        process_root_element(&start_tag, &mut fm_file_name, &mut saxml_version)?;
                        context.current_out_dir =
                            add_fm_name_to_base_path(root_out_dir, &fm_file_name, &saxml_version);
                        build_out_dir_path(&mut context, "", true);
                    }
                    1 => {
                        let do_continue = process_supporting_elements(&mut context, &start_tag);
                        if do_continue {
                            continue;
                        }
                    }
                    3 => {
                        let is_supported_catalog = process_catalog_elements(
                            &mut context,
                            &start_tag,
                            &saxml_version,
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
                // if depth <= 4 {
                //     println!("🚩 {}:{} {}", depth, "  ".repeat(depth), end_element_to_string(&e));
                // }
            }
            _ => {}
        }
        buf.clear()
    }

    // Write skeleton file if in lossless mode
    if flags.lossless {
        write_skeleton_file(&context.current_out_dir, context.skeleton, context.flags)?;
    }

    println!(
        "→ {} finished in {} ms.",
        fm_export_file_name,
        start.elapsed().as_millis()
    );

    Ok(())
}

fn process_root_element(
    e: &BytesStart,
    fm_file_name: &mut String,
    saxml_version: &mut String,
) -> Result<(), Error> {
    match e.name().as_ref() {
        b"FMDynamicTemplate" | b"FMSaveAsXML" => {
            *fm_file_name = get_attribute(e, "File")
                .unwrap()
                .strip_suffix(".fmp12")
                .unwrap()
                .to_string();
            *saxml_version = get_attribute(e, "version").unwrap().to_string();
        }
        _ => {
            bail!("Unsupported XML-format");
        }
    }
    Ok(())
}

/// Process supporting elements (Metadata, DDR_INFO) that are not part of the main structure
fn process_supporting_elements<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    start_tag: &BytesStart,
) -> bool {
    let mut do_continue = false;
    match start_tag.name().as_ref() {
        b"Metadata" => {
            process_supporting_element(context, start_tag, "metadata");
            do_continue = true;
        }
        b"DDR_INFO" => {
            process_supporting_element(context, start_tag, "ddr_info");
            do_continue = true;
        }
        _ => {}
    }
    do_continue
}

fn process_catalog_elements<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    start_tag: &BytesStart,
    saxml_version: &str,
    cf_folder_structure: &mut Option<FolderStructure>,
    script_folder_structure: &mut Option<FolderStructure>,
) -> Result<bool, Error> {
    let catalog_type = match CatalogType::from_bytes(start_tag.name().as_ref()) {
        Some(catalog_type) => catalog_type,
        None => {
            println!(
                "⚠️ Skipping unsupported catalog: {}",
                std::str::from_utf8(start_tag.name().as_ref()).unwrap()
            );
            return Ok(false);
        }
    };

    // Detect catalog type
    let mut catalog_config = catalog_type.get_config();
    // println!("             ✅ Processing catalog: {}, {}", start_element_to_string(e, flags), catalog_config.out_folder_name);

    // Handle special cases that need folder structures
    let folder_structure = match catalog_type {
        CatalogType::CalcsForCustomFunctions => cf_folder_structure.as_ref(),
        CatalogType::StepsForScripts => script_folder_structure.as_ref(),
        _ => None,
    };

    // Handle ValueList version-specific behavior
    if catalog_type == CatalogType::ValueList
        && version_string_to_number(saxml_version) < version_string_to_number("2.2.2.0")
    {
        catalog_config.out_folder_name = "value_list_options".to_string();
    }

    let xml_out_dir_path = build_out_dir_path(context, &catalog_config.out_folder_name, false);
    let base_dir_path = context.current_out_dir.clone();
    context.current_out_dir = xml_out_dir_path.clone();

    let folder_structure_result =
        xml_explode_catalog(context, start_tag, folder_structure, &catalog_config);

    context.current_out_dir = base_dir_path; // Restore to its original value

    // Update folder structures for catalogs that return them
    if let Some(folder_structure) = folder_structure_result {
        match catalog_type {
            CatalogType::CustomFunctions => *cf_folder_structure = Some(folder_structure),
            CatalogType::Script => *script_folder_structure = Some(folder_structure),
            _ => {}
        }
    }

    // Handle post-processing for StepsForScripts
    if catalog_type == CatalogType::StepsForScripts {
        let sanitized_scripts_dir_path = build_out_dir_path(context, "script_sanitized", false);
        create_sanitized_scripts(
            &xml_out_dir_path,
            &sanitized_scripts_dir_path,
            script_folder_structure.as_ref(),
            context.flags,
        );
    }
    Ok(true)
}

fn write_skeleton_file(
    skeleton_dir_path: &Path,
    skeleton: &Skeleton,
    flags: &Flags,
) -> Result<(), Error> {
    let skeleton_file_path = skeleton_dir_path.join("skeleton.xml");
    write_xml_file(&skeleton_file_path, &skeleton.content, 0, flags);
    Ok(())
}

fn push_start_to_skeleton(
    e: &BytesStart,
    skeleton: &mut Skeleton,
    path_stack: &[Vec<u8>],
    flags: &Flags,
) {
    if flags.lossless {
        match path_stack.len() {
            1..=3 => {
                push_line_to_skeleton(
                    skeleton,
                    path_stack.len(),
                    1,
                    start_element_to_string(e, flags).as_str(),
                    true,
                    XmlEventType::Start,
                );
            }
            4 => {
                if let Some(last) = path_stack.iter().rev().nth(1) {
                    let parent = std::str::from_utf8(last).unwrap();
                    if ["AddAction", "ModifyAction"].contains(&parent) {
                        push_line_to_skeleton(
                            skeleton,
                            path_stack.len(),
                            1,
                            start_element_to_string(e, flags).as_str(),
                            true,
                            XmlEventType::Start,
                        );
                    }
                }
            }
            5 => {
                if let Some(last) = path_stack.iter().rev().nth(2) {
                    let grandparent = std::str::from_utf8(last).unwrap();
                    if ["AddAction", "ModifyAction"].contains(&grandparent) {
                        push_line_to_skeleton(
                            skeleton,
                            path_stack.len(),
                            1,
                            start_element_to_string(e, flags).as_str(),
                            true,
                            XmlEventType::Start,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

fn push_end_to_skeleton(
    e: &BytesEnd,
    skeleton: &mut Skeleton,
    path_stack: &[Vec<u8>],
    flags: &Flags,
) {
    if flags.lossless {
        match path_stack.len() {
            0..=2 => {
                push_line_to_skeleton(
                    skeleton,
                    path_stack.len(),
                    1,
                    end_element_to_string(e).as_str(),
                    false,
                    XmlEventType::End,
                );
            }
            3 => {
                if let Some(last) = path_stack.iter().next_back() {
                    let parent = std::str::from_utf8(last).unwrap();
                    if ["AddAction", "ModifyAction"].contains(&parent) {
                        push_line_to_skeleton(
                            skeleton,
                            path_stack.len(),
                            1,
                            end_element_to_string(e).as_str(),
                            false,
                            XmlEventType::End,
                        );
                    }
                }
            }
            4 => {
                if let Some(last) = path_stack.iter().rev().nth(1) {
                    let grandparent = std::str::from_utf8(last).unwrap();
                    if ["AddAction", "ModifyAction"].contains(&grandparent) {
                        push_line_to_skeleton(
                            skeleton,
                            path_stack.len(),
                            1,
                            end_element_to_string(e).as_str(),
                            false,
                            XmlEventType::End,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}
