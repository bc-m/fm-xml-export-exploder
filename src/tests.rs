#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Read;
    use std::path::{Path, PathBuf};

    use insta::assert_snapshot;
    use similar_asserts::assert_eq;
    use walkdir::WalkDir;

    use crate::config::Flags;
    use crate::utils::file_utils::escape_filename;
    use crate::xml_processor::explode_xml;
    use crate::OutputTree;

    #[test]
    fn test_escape_filename() {
        assert_eq!(escape_filename("filename.xml"), "filename.xml");
        assert_eq!(escape_filename("file/name.xml"), "file_name.xml");
        assert_eq!(escape_filename("file|name.xml"), "file_name.xml");
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
        let path_str = if is_lossless {
            format!("./tests/out_{output_tree_str}_lossless")
        } else {
            format!("./tests/out_{output_tree_str}_lossy")
        };
        let output_dir = Path::new(&path_str);
        let flags = Flags {
            parse_all_lines: false,
            lossless: is_lossless,
            output_tree,
        };
        let _ = fs::remove_dir_all(output_dir);

        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(snapshot_dir);
        settings.set_prepend_module_to_snapshot(false);

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

    fn read_file(file_path: &PathBuf) -> Vec<u8> {
        let mut file = fs::File::open(file_path).expect("Failed to open file");
        let mut content = Vec::new();
        file.read_to_end(&mut content).expect("Failed to read file");
        content
    }
}
