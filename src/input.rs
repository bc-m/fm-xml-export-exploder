//! Finds the XML files to explode in the source directory.
//!
//! Besides complete exports (one file per database), FileMaker can split an export into
//! one file per catalog. It then writes a summary file (`<FMPReport type="Summary">`)
//! that refers to a folder per database holding the catalog files.

use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use encoding_rs_io::DecodeReaderBytes;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::utils::attributes::get_attribute;

/// The XML files in `source_dir`, with summary files of split exports replaced by the
/// catalog files of the folders they refer to.
pub fn input_files(source_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for path in xml_files_in(source_dir)? {
        match split_export_folders(&path)? {
            Some(folders) => {
                for folder in folders {
                    files.extend(xml_files_in(&folder)?);
                }
            }
            None => files.push(path),
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn xml_files_in(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(dir)
        .with_context(|| format!("Error reading directory {}", dir.display()))?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.is_file() && path.extension().is_some_and(|ext| ext == "xml")).then_some(path)
        })
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

/// If `path` is the summary file of a split export, the folders with the catalog files.
fn split_export_folders(path: &Path) -> Result<Option<Vec<PathBuf>>> {
    let file =
        File::open(path).with_context(|| format!("Error opening file {}", path.display()))?;
    let mut reader = Reader::from_reader(BufReader::new(DecodeReaderBytes::new(file)));
    let mut buf = Vec::new();
    let mut folders = None;
    loop {
        match reader.read_event_into(&mut buf) {
            // Not well-formed: leave it to the exploder to report the error
            Err(_) | Ok(Event::Eof) => break,
            Ok(Event::Start(e) | Event::Empty(e)) => match e.name().as_ref() {
                "FMPReport" if get_attribute(&e, "type").as_deref() == Some("Summary") => {
                    folders = Some(Vec::new());
                }
                "XML" if folders.is_some() => {
                    if let Some(xml_path) = get_attribute(&e, "path") {
                        let folder = resolve_folder(path, &xml_path).with_context(|| {
                            format!(
                                "Folder '{xml_path}' of split export {} not found",
                                path.display()
                            )
                        })?;
                        folders.as_mut().unwrap().push(folder);
                    }
                }
                // Any other root element: not a summary file
                _ if folders.is_none() => break,
                _ => {}
            },
            _ => {}
        }
        buf.clear();
    }
    Ok(folders)
}

/// The folder a summary file refers to. The summary holds the absolute path on the
/// machine that exported it, so a folder of the same name next to the summary file
/// is preferred: the export may have been moved or made on another machine.
fn resolve_folder(summary_path: &Path, xml_path: &str) -> Option<PathBuf> {
    let name = xml_path.rsplit(['/', '\\']).find(|part| !part.is_empty())?;
    let sibling = summary_path.parent()?.join(name);
    if sibling.is_dir() {
        return Some(sibling);
    }
    let absolute = PathBuf::from(xml_path);
    absolute.is_dir().then_some(absolute)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::input_files;

    #[test]
    fn summary_files_are_replaced_by_their_catalog_files() {
        let dir = std::env::temp_dir().join(format!(
            "fm-xml-export-exploder-input-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("Split")).unwrap();
        fs::write(
            dir.join("Summary.xml"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<FMPReport type="Summary" version="2.3.0.0" membercount="1">
	<File name="Split.fmp12" path="server.local">
		<XML path="/Volumes/Other Machine/exports/Split/"></XML>
	</File>
</FMPReport>
"#,
        )
        .unwrap();
        fs::write(dir.join("Complete.xml"), "<FMSaveAsXML></FMSaveAsXML>").unwrap();
        fs::write(dir.join("Split/Split_ScriptCatalog.xml"), "").unwrap();
        fs::write(dir.join("Split/Split_Metadata.xml"), "").unwrap();
        fs::write(dir.join("Split/notes.txt"), "").unwrap();

        let files = input_files(&dir).unwrap();
        let names: Vec<_> = files
            .iter()
            .map(|path| {
                path.strip_prefix(&dir)
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        assert_eq!(
            names,
            [
                "Complete.xml",
                "Split/Split_Metadata.xml",
                "Split/Split_ScriptCatalog.xml"
            ]
        );

        fs::remove_dir_all(&dir).unwrap();
    }
}
