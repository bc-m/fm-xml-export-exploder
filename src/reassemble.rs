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
use crate::utils::output_path::{
    SUPPORT_DOMAIN, build_tree_path, is_catalog_domain, output_roots, structure_folder_name,
};
use crate::utils::{
    LineState, create_dir, is_markup_line, state_after_line, version_string_to_number,
};
use crate::xml_processor::Action;

/// Reassembles every database in `source` into `target`, as `<database>.xml`, or as
/// `<database>_<catalog>.xml` for each file of an export split into catalogs.
/// Returns the item files that are not part of any skeleton.
pub fn reassemble(source: &Path, target: &Path) -> Result<Vec<PathBuf>> {
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

    let mut unused = Vec::new();
    for (db_name, output_tree) in &databases {
        let support_dir = build_tree_path(source, output_tree, SUPPORT_DOMAIN, db_name);
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
        unused.extend(items.unused(&catalog_folders(source, output_tree, db_name)?)?);
    }
    Ok(unused)
}

/// The catalog folders of a database, whether a skeleton refers to them or not
fn catalog_folders(source: &Path, output_tree: &OutputTree, db_name: &str) -> Result<Vec<PathBuf>> {
    let is_catalog = |domain_dir: Option<&Path>| {
        domain_dir
            .and_then(Path::file_name)
            .is_some_and(|name| is_catalog_domain(&name.to_string_lossy()))
    };
    let mut folders = Vec::new();
    for root in output_roots(source, output_tree, db_name) {
        match output_tree {
            // `<db>/<domain>`
            OutputTree::Db => folders.extend(
                subdirectories(&root)?
                    .into_iter()
                    .filter(|dir| is_catalog(Some(dir))),
            ),
            // `<domain>/<db>`
            OutputTree::Domain => {
                if is_catalog(root.parent()) {
                    folders.push(root);
                }
            }
        }
    }
    folders.sort();
    Ok(folders)
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
        if name == SUPPORT_DOMAIN {
            for db_dir in subdirectories(&dir)? {
                if !skeleton_files(&db_dir)?.is_empty() {
                    let db_name = db_dir.file_name().unwrap().to_string_lossy().to_string();
                    databases.push((db_name, OutputTree::Domain));
                }
            }
        } else if !skeleton_files(&dir.join(SUPPORT_DOMAIN))?.is_empty() {
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
                    "Metadata" => format!("metadata{}.xml", file.part),
                    _ => format!("ddr_info{}.xml", file.part),
                };
                // Lossless output has a file also for empty elements
                let path = file.support_dir.join(name);
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Cannot read {}", path.display()))?;
                push_indented(&mut out, "\t", &content);
                continue;
            }
            2 if !is_end => action = Action::from_name(tag),
            3 if is_end => catalog = None,
            3 => {
                let catalog_type = CatalogType::from_name(tag)
                    .filter(|catalog_type| catalog_type.is_exploded(ver));
                // The skeleton keeps only the outline of catalogs that aren't exploded
                if action.is_some() && catalog_type.is_none() && !is_whole_element(line) {
                    bail!("Unsupported catalog {tag}: its content is not in the output");
                }
                catalog = catalog_type.map(|catalog_type| {
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
        let mut placeholder = line.trim_start().to_string();
        if !is_whole_element(line) {
            let closing = format!("{}</{tag}>", &line[..depth]);
            while i < lines.len() && lines[i] != closing {
                placeholder.push('\n');
                placeholder.push_str(lines[i].trim_start());
                i += 1;
            }
            if i == lines.len() {
                bail!("No end tag for {}", line.trim_start());
            }
            i += 1;
            // The consumed lines end with the item's end tag
            state = LineState::Markup;
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

/// Whether the line holds the whole element, e.g. `<Catalog membercount="0"></Catalog>`
fn is_whole_element(line: &str) -> bool {
    let line = line.trim_start();
    line.ends_with("/>") || strip_end_tag(line).len() < line.len()
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
        let Some(files) = self.folders.get_mut(folder).unwrap().get_mut(key) else {
            return Ok(None);
        };
        // An id names one item, so more files would make the choice arbitrary. Items
        // without an id have it only in the reference lines after their start tag.
        let start_tag = key.lines().next().unwrap_or_default();
        if files.len() > 1 && attribute(start_tag, "id").is_some() {
            let paths: Vec<String> = files
                .iter()
                .map(|(path, _)| path.display().to_string())
                .collect();
            bail!(
                "{} matches several files: {}",
                key.replace('\n', " "),
                paths.join(", ")
            );
        }
        Ok(files.pop_front().map(|(_, content)| content))
    }

    /// The files in `folders` no placeholder asked for, e.g. leftovers of an older
    /// exploder version or the items of a catalog no skeleton refers to
    fn unused(&self, folders: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        for folder in folders {
            match self.folders.get(folder) {
                Some(files) => paths.extend(files.values().flatten().map(|(path, _)| path.clone())),
                None => paths.extend(xml_files_below(folder)?),
            }
        }
        paths.sort();
        Ok(paths)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "fm-xml-export-exploder-reassemble-unit-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Items without an id that share their reference are still taken one by one,
    /// although the reference has an id
    #[test]
    fn items_without_id_may_share_their_key() {
        let dir = temp_dir("shared-key");
        let item =
            "<Script>\n\t<ScriptReference id=\"5\" name=\"s\"></ScriptReference>\n</Script>\n";
        for sub in ["a", "b"] {
            fs::create_dir_all(dir.join(sub)).unwrap();
            fs::write(dir.join(sub).join("s.xml"), item).unwrap();
        }
        let config = KeyConfig {
            reference_tag: "ScriptReference",
            with_field: false,
        };
        let key = item_key(item, &config);

        let mut items = Items::default();
        assert!(items.take(&dir, &config, &key).unwrap().is_some());
        assert!(items.take(&dir, &config, &key).unwrap().is_some());
        assert!(items.take(&dir, &config, &key).unwrap().is_none());

        fs::remove_dir_all(&dir).unwrap();
    }

    /// A placeholder whose end tag is missing doesn't consume the rest of the skeleton
    #[test]
    fn placeholders_without_end_tag_are_an_error() {
        let dir = temp_dir("no-end-tag");
        let skeleton = "<FMSaveAsXML version=\"2.3.0.0\" File=\"T.fmp12\">\n\
            \t<Structure>\n\
            \t\t<AddAction>\n\
            \t\t\t<StepsForScripts membercount=\"1\">\n\
            \t\t\t\t<Script>\n\
            \t\t\t\t\t<ScriptReference id=\"5\" name=\"s\"></ScriptReference>\n\
            \t\t\t</StepsForScripts>\n\
            \t\t</AddAction>\n\
            \t</Structure>\n\
            </FMSaveAsXML>\n";
        let file = DatabaseFiles {
            source: &dir,
            output_tree: &OutputTree::Db,
            db_name: "T",
            support_dir: &dir.join("T/_"),
            part: "",
        };

        let err = reassemble_file(skeleton, &file, &mut Items::default()).unwrap_err();

        assert!(err.to_string().contains("No end tag for <Script>"), "{err}");
        fs::remove_dir_all(&dir).unwrap();
    }
}
