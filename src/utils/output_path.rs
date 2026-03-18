use std::fs;
use std::io::BufRead;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::config::{CatalogType, Flags, OutputTree};
use crate::xml_processor::{Action, ProcessingContext, Qualifier, TopLevelSection};

/// Version thresholds used for feature-gating catalog behavior.
pub(crate) const VERSION_2_2_2_0: u64 = version_string_to_number_const(2, 2, 2, 0);
pub(crate) const VERSION_2_2_3_4: u64 = version_string_to_number_const(2, 2, 3, 4);

/// Convert version string to number for comparison
/// For example, "20.3.1.2" -> 20003001002
/// "21" -> 21000000000
pub fn version_string_to_number(version: &str) -> u64 {
    let multipliers = [1_000_000_000, 1_000_000, 1_000, 1];
    version
        .split('.')
        .zip(multipliers)
        .map(|(s, m)| s.parse::<u64>().unwrap_or(0) * m)
        .sum()
}

/// Const-evaluable version of `version_string_to_number` for compile-time constants.
const fn version_string_to_number_const(major: u64, minor: u64, patch: u64, build: u64) -> u64 {
    major * 1_000_000_000 + minor * 1_000_000 + patch * 1_000 + build
}

pub fn build_out_dir_path<R: std::io::Read + BufRead>(
    context: &ProcessingContext<'_, R>,
    qualifier: Option<Qualifier>,
) -> Result<PathBuf> {
    let Some(doc_info) = &context.doc_info else {
        bail!("Missing document info (db name / version)");
    };
    let db_name = &doc_info.db_name;
    let ver = doc_info.saxml_version_num;

    let domain = match qualifier {
        Some(Qualifier::SanitizedScripts) => "scripts_sanitized".to_string(),
        Some(Qualifier::SanitizedCustomFunctions) => "custom_functions_sanitized".to_string(),
        None => resolve_structure_domain(context, ver),
    };

    Ok(build_tree_path(
        context.root_out_dir,
        &context.flags.output_tree,
        &domain,
        db_name,
    ))
}

/// Build the output directory path by joining the domain and db_name in the correct order
/// depending on the output tree mode.
fn build_tree_path(root: &Path, output_tree: &OutputTree, domain: &str, db_name: &str) -> PathBuf {
    match output_tree {
        OutputTree::Db => root.join(db_name).join(domain),
        OutputTree::Domain => root.join(domain).join(db_name),
    }
}

/// Resolve the domain folder name for the current processing context.
fn resolve_structure_domain<R: std::io::Read + BufRead>(
    context: &ProcessingContext<'_, R>,
    ver: u64,
) -> String {
    let Some(TopLevelSection::Structure) = context.top_level_section else {
        return "_".to_string();
    };

    let folder_name = match context.catalog_type {
        Some(CatalogType::ValueList) if (VERSION_2_2_2_0..VERSION_2_2_3_4).contains(&ver) => {
            "value_list_stubs"
        }
        Some(CatalogType::CustomFunctions) if ver >= VERSION_2_2_3_4 => "custom_functions",
        Some(catalog_type) => catalog_type.get_config().out_folder_name,
        None => "unknown_catalog",
    };

    let action_suffix = match &context.action {
        Some(Action::Add) | None => "",
        Some(Action::Modify) => "__modify_action",
        Some(Action::Replace) => "__replace_action",
        Some(Action::Delete) => "__delete_action",
    };

    format!("{folder_name}{action_suffix}")
}

pub fn delete_output_directory(context: &ProcessingContext<'_, impl BufRead>) -> Result<()> {
    let Some(doc_info) = &context.doc_info else {
        bail!("Missing document info (db name)");
    };
    let db_name = &doc_info.db_name;

    match context.flags.output_tree {
        OutputTree::Db => {
            let dir_to_delete = context.root_out_dir.join(db_name);
            if dir_to_delete.exists() {
                fs::remove_dir_all(&dir_to_delete)?;
            }
        }
        OutputTree::Domain => {
            if let Ok(entries) = fs::read_dir(context.root_out_dir) {
                for entry in entries.flatten() {
                    let target_dir = entry.path().join(db_name);
                    if target_dir.is_dir() {
                        fs::remove_dir_all(&target_dir)?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// In Domain mode, migrate old-format `custom_functions/` output (pre-2.2.3.4 style)
/// where it contained `.txt` files directly instead of `.xml` files.
/// Renames the entire `custom_functions/` -> `custom_functions_sanitized/` so git can track the rename.
pub fn migrate_old_custom_functions_if_needed(out_dir: &Path, flags: &Flags) -> Result<()> {
    if matches!(flags.output_tree, OutputTree::Db) {
        return Ok(());
    }

    let cf_dir = out_dir.join("custom_functions");
    let cf_sanitized_dir = out_dir.join("custom_functions_sanitized");

    if !cf_dir.exists() || cf_sanitized_dir.exists() {
        return Ok(());
    }

    let mut has_txt = false;
    let mut has_xml = false;
    fn check_dir(dir: &Path, has_txt: &mut bool, has_xml: &mut bool) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                check_dir(&path, has_txt, has_xml)?;
            } else if path.is_file() {
                match path.extension().and_then(|e| e.to_str()) {
                    Some("txt") => *has_txt = true,
                    Some("xml") => *has_xml = true,
                    _ => {}
                }
            }
        }
        Ok(())
    }
    check_dir(&cf_dir, &mut has_txt, &mut has_xml)?;

    if !has_txt || has_xml {
        return Ok(());
    }

    println!(
        "Migrating {} → {}",
        cf_dir.display(),
        cf_sanitized_dir.display()
    );
    fs::rename(&cf_dir, &cf_sanitized_dir)?;
    Ok(())
}
