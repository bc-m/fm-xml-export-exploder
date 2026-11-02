//! Reassembles the XML files of an export from output exploded with `--lossless`.
//!
//! The skeleton of each exploded file holds everything except the catalog items, and
//! for each item a placeholder: its start tag, or for items without an id the reference
//! that names them. Each placeholder is replaced by the item's file, with the
//! indentation the exploder removed added back.

use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::config::{CatalogType, OutputTree};
use crate::utils::file_utils::valid_dir_or_throw;
use crate::utils::output_path::{build_tree_path, structure_folder_name};
use crate::utils::{
    LineState, create_dir, is_markup_line, state_after_line, version_string_to_number,
};
use crate::xml_processor::Action;

/// Reassembles every database in `source` into `target`, as `<database>.xml`, or as
/// `<database>_<catalog>.xml` for each file of an export split into catalogs.
pub fn reassemble(source: &Path, target: &Path) -> Result<()> {
    valid_dir_or_throw(source)?;
    if std::path::absolute(target)?.starts_with(std::path::absolute(source)?) {
        bail!("The target directory must not be inside the source directory");
    }
    let databases = databases(source)?;
    if databases.is_empty() {
        bail!(
            "No skeleton files found in {}: reassembling needs output exploded with --lossless",
            source.display()
        );
    }
    create_dir(target);

    for (db_name, output_tree) in &databases {
        let support_dir = build_tree_path(source, output_tree, "_", db_name);
        let mut items = Items::default();
        for skeleton_path in skeleton_files(&support_dir)? {
            let stem = skeleton_path.file_stem().unwrap().to_string_lossy();
            // `skeleton_ScriptCatalog` belongs to `<database>_ScriptCatalog.xml`
            let part = stem.strip_prefix("skeleton").unwrap_or_default();
            let skeleton = fs::read_to_string(&skeleton_path)
                .with_context(|| format!("Cannot read {}", skeleton_path.display()))?;
            let file = DatabaseFiles {
                source,
                output_tree,
                db_name,
                support_dir: &support_dir,
                part,
            };
            let xml = reassemble_file(&skeleton, &file, &mut items)
                .with_context(|| format!("Cannot reassemble {}", skeleton_path.display()))?;
            let out_path = target.join(format!("{db_name}{part}.xml"));
            fs::write(&out_path, xml)
                .with_context(|| format!("Cannot write {}", out_path.display()))?;
            println!("→ {}", out_path.display());
        }
        for path in items.unused() {
            eprintln!("Not part of any skeleton: {}", path.display());
        }
    }
    Ok(())
}

/// Where the exploded files of one database are
struct DatabaseFiles<'a> {
    source: &'a Path,
    output_tree: &'a OutputTree,
    db_name: &'a str,
    /// The `_` folder with the skeletons, metadata and DDR_INFO
    support_dir: &'a Path,
    /// The catalog part of a split export, e.g. `_ScriptCatalog`, or empty
    part: &'a str,
}

/// The databases in `source`, with the output tree they were exploded with
fn databases(source: &Path) -> Result<Vec<(String, OutputTree)>> {
    let mut databases = Vec::new();
    for dir in subdirectories(source)? {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        if name == "_" {
            for db_dir in subdirectories(&dir)? {
                if !skeleton_files(&db_dir)?.is_empty() {
                    let db_name = db_dir.file_name().unwrap().to_string_lossy().to_string();
                    databases.push((db_name, OutputTree::Domain));
                }
            }
        } else if !skeleton_files(&dir.join("_"))?.is_empty() {
            databases.push((name, OutputTree::Db));
        }
    }
    Ok(databases)
}

fn subdirectories(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("Cannot read {}", dir.display()))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    Ok(dirs)
}

fn skeleton_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(Vec::new());
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "xml")
                && path
                    .file_stem()
                    .is_some_and(|stem| stem.to_string_lossy().starts_with("skeleton"))
        })
        .collect();
    files.sort();
    Ok(files)
}

/// The catalog the skeleton lines currently belong to
struct Catalog {
    folder: PathBuf,
    item_name: &'static str,
    /// Indentation of the catalog items: 4, or 5 when wrapped in an `ObjectList`
    item_depth: usize,
    key: KeyConfig,
}

fn reassemble_file(skeleton: &str, file: &DatabaseFiles, items: &mut Items) -> Result<String> {
    let lines: Vec<&str> = skeleton.split_terminator('\n').collect();
    let ver = lines
        .iter()
        .find_map(|line| attribute(line, "version").filter(|_| line.starts_with("<FM")))
        .map_or(0, |version| version_string_to_number(&version));

    let mut out = String::with_capacity(skeleton.len() * 16);
    let mut state = LineState::Markup;
    let mut action = None;
    let mut catalog: Option<Catalog> = None;
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let is_markup = is_markup_line(line, state);
        state = state_after_line(line, state);
        i += 1;
        if !is_markup {
            push_line(&mut out, line);
            continue;
        }
        let depth = line.len() - line.trim_start_matches('\t').len();
        let tag = tag_name(line.trim_start());
        let is_end = line.trim_start().starts_with("</");

        match depth {
            1 if !is_end && matches!(tag, "Metadata" | "DDR_INFO") => {
                let name = match tag {
                    "Metadata" => "metadata.xml".to_string(),
                    _ => format!("ddr_info{}.xml", file.part),
                };
                // Empty elements were not written to a file
                match fs::read_to_string(file.support_dir.join(name)) {
                    Ok(content) => push_indented(&mut out, "\t", &content),
                    Err(_) => push_line(&mut out, line),
                }
                continue;
            }
            2 if !is_end => action = Action::from_name(tag),
            3 if is_end => catalog = None,
            3 => {
                catalog = CatalogType::from_name(tag).map(|catalog_type| {
                    let config = catalog_type.get_config();
                    let folder_name =
                        structure_folder_name(Some(catalog_type), ver, action.as_ref());
                    Catalog {
                        folder: build_tree_path(
                            file.source,
                            file.output_tree,
                            &folder_name,
                            file.db_name,
                        ),
                        item_name: config.catalog_item_name,
                        item_depth: if config.wrapped_in_object_list { 5 } else { 4 },
                        key: KeyConfig {
                            reference_tag: config.id_path.rsplit('/').nth(1).unwrap_or_default(),
                            with_field: tag == "FieldsForTables"
                                && matches!(action, Some(Action::Modify)),
                        },
                    }
                });
            }
            _ => {}
        }

        let Some(catalog) = catalog
            .as_ref()
            .filter(|c| depth == c.item_depth && !is_end && tag == c.item_name)
        else {
            push_line(&mut out, line);
            continue;
        };

        // The placeholder is one line, or the item's reference lines up to its end tag
        let end_tag = format!("</{tag}>");
        let mut placeholder = line.trim_start().to_string();
        if !line.ends_with(&end_tag) {
            let closing = format!("{}{end_tag}", &line[..depth]);
            while i < lines.len() && lines[i] != closing {
                placeholder.push('\n');
                placeholder.push_str(lines[i].trim_start());
                i += 1;
            }
            i += 1;
        }
        let key = item_key(&placeholder, &catalog.key);
        let content = items
            .take(&catalog.folder, &catalog.key, &key)?
            .with_context(|| {
                format!(
                    "No file for {} in {}",
                    key.replace('\n', " "),
                    catalog.folder.display()
                )
            })?;
        push_indented(&mut out, &line[..depth], &content);
    }
    Ok(out)
}

fn push_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push('\n');
}

/// Adds the content of an exploded file, indenting the lines the exploder unindented
fn push_indented(out: &mut String, indent: &str, content: &str) {
    let mut state = LineState::Markup;
    for line in content.split_terminator('\n') {
        if is_markup_line(line, state) {
            out.push_str(indent);
        }
        state = state_after_line(line, state);
        push_line(out, line);
    }
}

/// How an item without an id is named in its placeholder
struct KeyConfig {
    /// The element with the id, e.g. `ScriptReference` for script steps
    reference_tag: &'static str,
    /// Modified fields share the table reference and are also named by their field
    with_field: bool,
}

/// What identifies an item, in its file and in its placeholder: the start tag, and for
/// items without an id the start tags of the references that name them
fn item_key(xml: &str, config: &KeyConfig) -> String {
    let first = xml.lines().next().unwrap_or_default().trim_start();
    let mut key = strip_end_tag(first).to_string();
    if attribute(&key, "id").is_some() || config.reference_tag.is_empty() {
        return key;
    }
    let field_tag = config.with_field.then_some("FieldReference");
    for reference in [Some(config.reference_tag), field_tag]
        .into_iter()
        .flatten()
    {
        if let Some(line) = xml
            .lines()
            .map(str::trim_start)
            .find(|line| tag_name(line) == reference)
        {
            key.push('\n');
            key.push_str(strip_end_tag(line));
        }
    }
    key
}

/// The item files of the catalog folders read so far, by their key
#[derive(Default)]
struct Items {
    folders: HashMap<PathBuf, HashMap<String, VecDeque<(PathBuf, String)>>>,
}

impl Items {
    fn take(&mut self, folder: &Path, config: &KeyConfig, key: &str) -> Result<Option<String>> {
        if !self.folders.contains_key(folder) {
            let mut files = HashMap::<String, VecDeque<(PathBuf, String)>>::new();
            for path in xml_files_below(folder)? {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Cannot read {}", path.display()))?;
                files
                    .entry(item_key(&content, config))
                    .or_default()
                    .push_back((path, content));
            }
            self.folders.insert(folder.to_path_buf(), files);
        }
        Ok(self
            .folders
            .get_mut(folder)
            .unwrap()
            .get_mut(key)
            .and_then(VecDeque::pop_front)
            .map(|(_, content)| content))
    }

    /// Item files no placeholder asked for, e.g. leftovers of an older exploder version
    fn unused(&self) -> Vec<&Path> {
        let mut paths: Vec<&Path> = self
            .folders
            .values()
            .flat_map(|files| files.values().flatten())
            .map(|(path, _)| path.as_path())
            .collect();
        paths.sort();
        paths
    }
}

fn xml_files_below(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(files);
    };
    for path in entries.flatten().map(|entry| entry.path()) {
        if path.is_dir() {
            files.extend(xml_files_below(&path)?);
        } else if path.extension().is_some_and(|ext| ext == "xml") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// The element name of a start or end tag, e.g. `Script` for `<Script id="1">`
fn tag_name(tag: &str) -> &str {
    let name = tag.trim_start_matches('<').trim_start_matches('/');
    let end = name.find([' ', '>', '/']).unwrap_or(name.len());
    &name[..end]
}

/// `<Script id="1"></Script>` without its end tag
fn strip_end_tag(line: &str) -> &str {
    let end_tag = format!("</{}>", tag_name(line));
    line.strip_suffix(end_tag.as_str()).unwrap_or(line)
}

/// The raw value of an attribute in the start tag of a line
fn attribute(line: &str, name: &str) -> Option<String> {
    let start = line.find(&format!(" {name}=\""))? + name.len() + 3;
    let len = line[start..].find('"')?;
    Some(line[start..start + len].to_string())
}
