# FileMaker XML-Export File Exploder

FileMaker XML-Export File Exploder is a fast Rust tool designed to parse XML files exported from FileMaker databases and
extract relevant content into separate files. It efficiently processes a directory of new FileMaker XML report files,
splitting the content by its purpose (such as scripts, layouts, custom functions, and table definitions), and applies
the directory structure from FileMaker scripts and layouts.

## Features

- **Fast Processing:** Built in Rust for high performance and efficiency.
- **Parallel Processing:** Utilizes parallel processing for efficient extraction, thanks to the Rayon crate.
- **Content Extraction:** Parses XML files and extracts relevant content into separate files.
- **Directory Structure:** Organizes extracted content into directories mirroring the structure in FileMaker databases.
- **Noise Reduction:** Removes unnecessary noise from XML to support efficient Git versioning and prevent unnecessary
  changes.
- **Human-Readable Format:** Parses scripts and custom functions into a human-readable format for easy comprehension.

## Installation

1. **Download Executable**: Download the latest executable for your operating system from
   the [releases page](https://github.com/BC-M/fm-xml-export-exploder/releases/latest).
2. __macOS only:__ To allow execution in Terminal, right-click on the executable and choose "Open" to bypass the
   security warning. After this, you can run the CLI in Terminal or Bash scripts.

## Usage

1. **Input Directory**: Provide the path to the directory containing the new FileMaker XML report files.
2. **Output Directory**: Specify the directory where the extracted content will be saved.
3. **Run the Tool**: Execute the tool by running `fm-xml-export-exploder [INPUT_DIRECTORY] [OUTPUT_DIRECTORY]`.

## Output Organization

The extracted content is organized into directories based on the context of the XML elements:

```bash
├── custom_functions
│   └── [FileMaker Database Name]
│       └── [CF Name] - ID [CF ID].txt
├── external_data_sources
│   └── [FileMaker Database Name].xml
├── layouts
│   └── [FileMaker Database Name]
│       └── [Directory name] - ID [Directory ID]
│           └── [Layout name] - ID [Layout ID].xml
├── relationships
│   └── [FileMaker Database Name]
│       └── [Left Table name] - [Right Table name] - ID [Relationship ID].xml
├── scripts
│   └── [FileMaker Database Name]
│       └── [Directory name] - ID [Directory ID]
│           └── [Script name] - ID [Script ID].xml
├── scripts_sanitized
│   └── [FileMaker Database Name]
│       └── [Directory name] - ID [Directory ID]
│           └── [Script name] - ID [Script ID].txt
├── tables
│   └── [FileMaker Database Name]
│       └── [Table name] - ID [Table ID].xml
├── themes
│   └── [FileMaker Database Name]
│       └── [Theme name] - ID [Theme ID].xml
└── value_lists
   └── [FileMaker Database Name]
        └── [Value list name] - ID [Value list ID].xml
```

For multi-file solutions it can be helpful to create a separate Git repository for each of these
directories (`custom_functions`, `external_data_sources`, `layouts`, `scripts`, `scripts_sanitized`, `tables`, `themes`, `value_lists`)
to manage version control and collaboration effectively.

## Why this structure?

Well, git does not work very well with large files or large *number* of files. In our case, we mainly need to identify
changes in scripts, so we decided to use this structure to better review script changes in `scripts_sanitized` in a
human-readable format like scripts displayed in FileMaker. If you have a small solution with a small number of FileMaker
files, you can also push all the files to a git repository.

## Best practise

- Create a FileMaker script to automatically export your solution file(s) as XML using the [Save Copy as XML
  command](https://help.claris.com/en/pro-help/content/save-a-copy-as-xml.html). __Tip:__ You can apply the command to
  any opened FileMaker file that is open with full access by window name.
- Initialize a git repository for each [directory](#output-organization) created.
- Extend your FileMaker script to run `git add --all`, `git commit -m "COMMIT MESSAGE HERE"` and `git push`.

Congratulations, now your FileMaker changes are versioned using git!

## Dependencies

- [xml](https://crates.io/crates/xml): Rust crate for XML parsing.
- [rayon](https://crates.io/crates/rayon): Rust crate for parallelism.
- [encoding_rs_io](https://crates.io/crates/encoding_rs_io): Rust crate for character encoding support.

## License

This tool is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Feel free to open issues or pull requests on
the [GitHub repository](https://github.com/BC-M/fm-xml-export-exploder).

## TODOs

- [ ] Parse unknown script steps:
    - [ ] TODO: List all script steps here
- [ ] Parse content of FileMaker XML-Export contents:
    - [x] ExternalDataSourceCatalog
    - [ ] BaseTableCatalog
    - [ ] TableOccurrenceCatalog
    - [ ] CustomFunctionsCatalog
    - [x] FieldsForTables
    - [x] ValueListCatalog
    - [x] RelationshipCatalog
    - [x] CalcsForCustomFunctions
    - [x] ScriptCatalog
    - [x] ThemeCatalog
    - [x] LayoutCatalog
    - [ ] PrivilegeSetsCatalog
    - [ ] ExtendedPrivilegesCatalog
    - [ ] AccountsCatalog
    - [x] StepsForScripts
    - [ ] CustomMenuCatalog
    - [ ] CustomMenuSetCatalog
