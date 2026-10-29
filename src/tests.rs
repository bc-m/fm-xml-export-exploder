#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use std::fs;
    use std::io::Read;
    use std::path::{Path, PathBuf};

    use encoding_rs_io::DecodeReaderBytes;
    use insta::assert_snapshot;
    use similar_asserts::assert_eq;
    use walkdir::WalkDir;

    use crate::config::{Flags, OutputTree};
    use crate::reassemble::reassemble;
    use crate::utils::file_utils::escape_filename;
    use crate::xml_processor::explode_xml;

    #[test]
    fn test_escape_filename() {
        assert_eq!(escape_filename("filename.xml"), "filename.xml");
        assert_eq!(escape_filename("file/name.xml"), "file_name.xml");
        assert_eq!(escape_filename("file|name.xml"), "file_name.xml");
    }

    /// Modified layouts are named after their layout reference and still placed in
    /// the folder they belong to.
    #[test]
    fn content_named_items_are_placed_in_their_folder() {
        let dir = std::env::temp_dir().join(format!(
            "fm-xml-export-exploder-folders-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let input = dir.join("t.xml");
        fs::write(
            &input,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<FMSaveAsXML version="2.2.1.0" File="T.fmp12">
	<Structure>
		<ModifyAction>
			<LayoutCatalog membercount="1">
				<Layout isFolder="True" id="1" name="Folder"></Layout>
				<Layout>
					<LayoutReference id="9" name="Home"></LayoutReference>
				</Layout>
				<Layout isFolder="Marker" id="1"></Layout>
			</LayoutCatalog>
		</ModifyAction>
	</Structure>
</FMSaveAsXML>
"#,
        )
        .unwrap();
        let flags = Flags {
            parse_all_lines: false,
            lossless: false,
            output_tree: OutputTree::Db,
            obfuscate_passwords: false,
            script_extension: "txt".to_string(),
        };

        let output_dir = dir.join("out");
        explode_xml(&input, &output_dir, &flags).unwrap();

        assert!(
            output_dir
                .join("T/layouts__modify_action/Folder - ID 1/Home - ID 9.xml")
                .exists()
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    /// Persistent stores (FileMaker 2026) get a file each, with their value
    #[test]
    fn persistent_stores() {
        let dir = std::env::temp_dir().join(format!(
            "fm-xml-export-exploder-persistent-stores-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let input = dir.join("t.xml");
        fs::write(
            &input,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<FMSaveAsXML version="2.3.0.0" File="T.fmp12">
	<Structure>
		<AddAction>
			<PersistentStoreCatalog membercount="1">
				<PersistentStore name="pi" instanceID="precision_17_digits" id="1">
					<Value type="Number">
						<StyledText>
							<Data><![CDATA[3.1415926535897932]]></Data>
						</StyledText>
					</Value>
				</PersistentStore>
			</PersistentStoreCatalog>
		</AddAction>
	</Structure>
</FMSaveAsXML>
"#,
        )
        .unwrap();
        let flags = Flags {
            parse_all_lines: false,
            lossless: false,
            output_tree: OutputTree::Db,
            obfuscate_passwords: false,
            script_extension: "txt".to_string(),
        };

        let output_dir = dir.join("out");
        explode_xml(&input, &output_dir, &flags).unwrap();

        let store =
            fs::read_to_string(output_dir.join("T/persistent_stores/pi - ID 1.xml")).unwrap();
        assert!(store.contains("<Data><![CDATA[3.1415926535897932]]></Data>"));
        fs::remove_dir_all(&dir).unwrap();
    }

    /// Reassembling lossless output gives back the exported files, in both output trees
    #[test]
    fn reassembled_exports_equal_the_originals() {
        for (output_tree, name) in [(OutputTree::Db, "db"), (OutputTree::Domain, "domain")] {
            let dir = std::env::temp_dir().join(format!(
                "fm-xml-export-exploder-reassemble-{name}-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&dir);
            let flags = Flags {
                parse_all_lines: false,
                lossless: true,
                output_tree,
                obfuscate_passwords: false,
                script_extension: "txt".to_string(),
            };
            let mut exports = Vec::new();
            for entry in fs::read_dir("tests/xml").unwrap() {
                let path = entry.unwrap().path();
                let db_name = explode_xml(&path, &dir.join("out"), &flags)
                    .unwrap()
                    .unwrap();
                exports.push((db_name, path));
            }

            reassemble(&dir.join("out"), &dir.join("xml")).unwrap();

            for (db_name, original) in exports {
                let mut expected = String::new();
                DecodeReaderBytes::new(fs::File::open(&original).unwrap())
                    .read_to_string(&mut expected)
                    .unwrap();
                let actual =
                    fs::read_to_string(dir.join("xml").join(format!("{db_name}.xml"))).unwrap();
                // A plain assert: a diff of these files would be too large to read
                assert!(
                    expected.trim_start_matches('\u{feff}') == actual,
                    "{} ({name}) differs after reassembling",
                    original.display()
                );
            }
            fs::remove_dir_all(&dir).unwrap();
        }
    }

    /// Without the skeletons of lossless output, nothing can be reassembled
    #[test]
    fn reassembling_needs_lossless_output() {
        let dir = std::env::temp_dir().join(format!(
            "fm-xml-export-exploder-reassemble-lossy-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let flags = Flags {
            parse_all_lines: false,
            lossless: false,
            output_tree: OutputTree::Db,
            obfuscate_passwords: false,
            script_extension: "txt".to_string(),
        };
        let input = PathBuf::from("tests/xml/Ooe - saxml_v_2_2_3_4.xml");
        explode_xml(&input, &dir.join("out"), &flags).unwrap();

        let err = reassemble(&dir.join("out"), &dir.join("xml")).unwrap_err();
        assert!(err.to_string().contains("--lossless"), "{err}");
        assert!(!dir.join("xml").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    /// Readable scripts get the configured extension, custom functions keep `.txt`, and
    /// scripts written with the previous extension are removed on the next run.
    #[test]
    fn script_extension() {
        let dir = std::env::temp_dir().join(format!(
            "fm-xml-export-exploder-extension-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let input = PathBuf::from("tests/xml/Ooe - saxml_v_2_2_3_4.xml");
        let flags = |script_extension: &str| Flags {
            parse_all_lines: false,
            lossless: false,
            output_tree: OutputTree::Db,
            obfuscate_passwords: false,
            script_extension: script_extension.to_string(),
        };
        let extensions_in = |folder: &str| -> Vec<String> {
            let mut extensions: Vec<String> = WalkDir::new(dir.join("Ooe").join(folder))
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_file())
                .filter_map(|entry| Some(entry.path().extension()?.to_str()?.to_string()))
                .collect();
            extensions.sort();
            extensions.dedup();
            extensions
        };

        explode_xml(&input, &dir, &flags("txt")).unwrap();
        assert_eq!(extensions_in("scripts_sanitized"), ["txt"]);

        explode_xml(&input, &dir, &flags("fmscript")).unwrap();
        assert_eq!(extensions_in("scripts_sanitized"), ["fmscript"]);
        assert_eq!(extensions_in("custom_functions_sanitized"), ["txt"]);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn snapshot_test_lossless() {
        snapshot_test_with_mode(true, OutputTree::Db, "snapshots_db_lossless");
    }

    #[test]
    fn snapshot_test_lossy_domain() {
        snapshot_test_with_mode(false, OutputTree::Domain, "snapshots_domain_lossy");
    }

    fn snapshot_test_with_mode(
        is_lossless: bool,
        output_tree: OutputTree,
        snapshot_folder_name: &str,
    ) {
        let path_str = format!("./tests/{}", snapshot_folder_name);
        let snapshot_dir = Path::new(&path_str);
        let input_dir = Path::new("./tests/xml");
        // Make output_dir mode-specific to avoid interference from prior or parallel test runs
        let output_tree_str = match output_tree {
            OutputTree::Db => "db",
            OutputTree::Domain => "domain",
        };
        let lossless_str = if is_lossless { "lossless" } else { "lossy" };
        let path_str = format!("./tests/out_{output_tree_str}_{lossless_str}");
        let output_dir = Path::new(&path_str);
        let flags = Flags {
            parse_all_lines: false,
            lossless: is_lossless,
            output_tree,
            obfuscate_passwords: false,
            script_extension: "txt".to_string(),
        };
        let _ = fs::remove_dir_all(output_dir);

        let paths = fs::read_dir(input_dir)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.is_file())
            .filter(|e| e.file_name().unwrap() != ".DS_Store")
            .collect::<Vec<_>>();

        for path in paths {
            explode_xml(&path, output_dir, &flags)
                .unwrap_or_else(|_| panic!("Error processing file '{}'", path.display()));
        }

        let output_files: Vec<PathBuf> = WalkDir::new(output_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| e.file_name().to_str().unwrap() != ".DS_Store")
            .map(|e| e.path().to_path_buf())
            .collect();

        for output_file in &output_files {
            let output_content = String::from_utf8(read_file(output_file)).unwrap();
            let output_file_slug = output_file.strip_prefix(output_dir).unwrap();
            let path_str = format!("../tests/{}", snapshot_folder_name);
            let snapshot_path = Path::new(&path_str)
                .join(output_file_slug)
                .parent()
                .unwrap()
                .to_path_buf();

            // Use fresh Settings scope for each snapshot to avoid interference from prior or parallel test runs
            insta::with_settings!({
                snapshot_path => snapshot_path,
                prepend_module_to_snapshot => false,
            }, {
                assert_snapshot!(output_file.file_name().unwrap().to_str(), output_content);
            });
        }

        let mut output_file_paths = output_files
            .iter()
            .map(|file| file.strip_prefix(output_dir).unwrap())
            .map(|file| file.to_string_lossy())
            .collect::<Vec<_>>();
        output_file_paths.sort();

        let snapshot_files: Vec<PathBuf> = WalkDir::new(snapshot_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| e.file_name().to_str().unwrap().ends_with(".snap"))
            .map(|e| e.path().to_path_buf())
            .collect();

        let mut snapshot_file_paths = snapshot_files
            .iter()
            .map(|file| file.strip_prefix(snapshot_dir).unwrap())
            .map(|file| {
                file.to_string_lossy()
                    .strip_suffix(".snap")
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<_>>();
        snapshot_file_paths.sort();

        assert_eq!(snapshot_file_paths.join("\n"), output_file_paths.join("\n"));
    }

    fn read_file(file_path: &Path) -> Vec<u8> {
        fs::read(file_path).expect("Failed to read file")
    }
}
