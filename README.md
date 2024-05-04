# FileMaker XML-Export File Exploder

FileMaker XML-Export File Exploder is a fast Rust tool designed to parse XML files exported from FileMaker databases and extract relevant content into separate files. It efficiently processes a directory of new FileMaker XML report files, splitting the content by its purpose (such as scripts, layouts, custom functions, and table definitions), and applies the directory structure from FileMaker scripts and layouts.

## TL;DR:

Do you want to extract your FileMaker solution into text files (including scripts in a human-readable format like the FileMaker Script Workspace)? This tool does it for you!

Example: From [these](./tests/xml) as XML exported FileMaker solutions you will get [this](./tests/snapshots). Scripts can easily be read like [here](./tests/snapshots/scripts_sanitized/fmSyntaxColorizer/Example%20SyntaxColorizing%20-%20ID%2046/All%20script%20steps%20and%20all%20options%20-%20ID%2011.txt.snap).

## Features

- **Fast Processing:** Built in Rust for high performance and efficiency.
- **Parallel Processing:** Utilizes parallel processing for efficient extraction, thanks to the Rayon crate.
- **Content Extraction:** Parses XML files and extracts relevant content into separate files and organizes them into directories mirroring the structure in FileMaker databases.
- **Noise Reduction:** Removes unnecessary noise from XML to support efficient Git versioning and prevent unnecessary changes.
- **Human-Readable Format:** Parses scripts and custom functions into a human-readable format for easy comprehension.

## Installation

1. **Download Executable:** Download the latest executable for your operating system from the [releases page](https://github.com/BC-M/fm-xml-export-exploder/releases/latest).
2. __macOS only:__ To allow execution in Terminal, right-click on the executable and choose "Open" to bypass the security warning. After this, you can run the CLI in Terminal or Bash scripts.

## Usage

1. **Input Directory:** Provide the path to the directory containing the new FileMaker XML report files.
2. **Output Directory:** Specify the directory where the extracted content will be saved.
3. **Run the Tool:** Execute the tool by running `fm-xml-export-exploder [INPUT_DIRECTORY] [OUTPUT_DIRECTORY]`.

## Output Organization

The extracted content is organized into directories based on the context of the XML elements:

```bash
├── custom_functions
│   └── [FileMaker database name]
│       └── [CF name] - ID [CF ID].txt
├── custom_menu_sets
│   └── [FileMaker database name]
│       └── [Set name] - ID [Set ID].txt
├── custom_menus
│   └── [FileMaker database name]
│       └── [Menu name] - ID [Menu ID].txt
├── extended_privileges
│   └── [FileMaker database name]
│       └── [Privileges name] - ID [Privileges ID].txt
├── external_data_sources
│   └── [FileMaker database name].xml
├── layouts
│   └── [FileMaker database name]
│       └── [Directory name] - ID [Directory ID]
│           └── [Layout name] - ID [Layout ID].xml
├── privilege_sets
│   └── [FileMaker database name]
│       └── [Set name] - ID [Set ID].txt
├── relationships
│   └── [FileMaker database name]
│       └── [Left Table name] - [Right Table name] - ID [Relationship ID].xml
├── scripts
│   └── [FileMaker database name]
│       └── [Directory name] - ID [Directory ID]
│           └── [Script name] - ID [Script ID].xml
├── scripts_sanitized
│   └── [FileMaker database name]
│       └── [Directory name] - ID [Directory ID]
│           └── [Script name] - ID [Script ID].txt
├── tables
│   └── [FileMaker database name]
│       └── [Table name] - ID [Table ID].xml
├── table_occurrences
│   └── [FileMaker database name]
│       └── [TO name] - ID [TO ID].xml
├── themes
│   └── [FileMaker database name]
│       └── [Theme name] - ID [Theme ID].xml
└── value_lists
    └── [FileMaker database name]
        └── [Value list name] - ID [Value list ID].xml
```

For multi-file solutions it can be helpful to create a separate Git repository for each of these directories (`custom_functions`, `layouts`, `scripts`, `tables` and so on) to manage version control and collaboration effectively.

## Why this structure?

Well, git does not work very well with large files or large *number* of files. In our case, we mainly need to track changes in scripts, so we decided to use this structure to better review script changes in `scripts_sanitized` in a more human-readable format like scripts displayed in FileMaker. If you have a small solution with a small number of FileMaker files, you can also push all the files to a git repository.

## Best practise

- Create a FileMaker script to automatically export your solution file(s) as XML using the [Save Copy as XML command](https://help.claris.com/en/pro-help/content/save-a-copy-as-xml.html). __Tip:__ You can apply the command to any opened FileMaker file that is open with full access by window name.
- Initialize a git repository for each [directory](#output-organization) created.
- Extend your FileMaker script to run `git add --all`, `git commit -m "COMMIT MESSAGE HERE"` and `git push`.

Congratulations, now your FileMaker changes are versioned using git!

## Imported FileMaker solutions

The following open source FileMaker solutions were saved as XML and used for snapshot tests.

- [FM-Admin-API-Tool](https://github.com/SoliantMike/FM-Admin-API-Tool): A FileMaker tool for administering FileMaker Server using the Admin API.
- [fmSyntaxColorizer](https://github.com/mrwatson-de/fmSyntaxColorizer): A FileMaker tool for customizing syntax highlighting for FileMaker scripts and calculations.

## Dependencies

- [anyhow](https://crates.io/crates/anyhow): Rust crate for flexible error handling.
- [clap](https://crates.io/crates/clap): Rust crate for command-line argument parsing.
- [encoding_rs_io](https://crates.io/crates/encoding_rs_io): Rust crate for character encoding support.
- [quick-xml](https://crates.io/crates/quick-xml): Rust crate for high-performance XML parsing.
- [rayon](https://crates.io/crates/rayon): Rust crate for parallelism.
- [regex](https://crates.io/crates/regex): Rust crate for regular expressions.
- [strum](https://crates.io/crates/strum): Rust crate for easier management of enums and strings.

## License

This tool is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Feel free to open issues or pull requests on the [GitHub repository](https://github.com/BC-M/fm-xml-export-exploder).
