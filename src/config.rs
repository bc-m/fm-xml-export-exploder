use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogType {
    Accounts,
    BaseDirectory,
    BaseTable,
    CalcsForCustomFunctions,
    CustomFunctions,
    CustomMenu,
    CustomMenuSet,
    ExternalDataSource,
    ExtendedPrivileges,
    FieldsForTables,
    FileAccess,
    Layout,
    Library,
    OptionsForValueLists,
    PrivilegeSets,
    Relationship,
    Script,
    StepsForScripts,
    TableOccurrence,
    Theme,
    ValueList,
}

impl CatalogType {
    pub const fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"AccountsCatalog" => Some(Self::Accounts),
            b"BaseDirectoryCatalog" => Some(Self::BaseDirectory),
            b"BaseTableCatalog" => Some(Self::BaseTable),
            b"CalcsForCustomFunctions" => Some(Self::CalcsForCustomFunctions),
            b"CustomFunctionsCatalog" => Some(Self::CustomFunctions),
            b"CustomMenuCatalog" => Some(Self::CustomMenu),
            b"CustomMenuSetCatalog" => Some(Self::CustomMenuSet),
            b"ExternalDataSourceCatalog" => Some(Self::ExternalDataSource),
            b"ExtendedPrivilegesCatalog" => Some(Self::ExtendedPrivileges),
            b"FieldsForTables" => Some(Self::FieldsForTables),
            b"FileAccessCatalog" => Some(Self::FileAccess),
            b"LayoutCatalog" => Some(Self::Layout),
            b"LibraryCatalog" => Some(Self::Library),
            b"OptionsForValueLists" => Some(Self::OptionsForValueLists),
            b"PrivilegeSetsCatalog" => Some(Self::PrivilegeSets),
            b"RelationshipCatalog" => Some(Self::Relationship),
            b"ScriptCatalog" => Some(Self::Script),
            b"StepsForScripts" => Some(Self::StepsForScripts),
            b"TableOccurrenceCatalog" => Some(Self::TableOccurrence),
            b"ThemeCatalog" => Some(Self::Theme),
            b"ValueListCatalog" => Some(Self::ValueList),
            _ => None,
        }
    }

    pub const fn get_config(&self) -> CatalogConfig {
        match self {
            Self::Accounts => CatalogConfig {
                catalog_item_name: b"Account",
                out_folder_name: "accounts",
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "Account/AccountReference/@id",
            },
            Self::BaseDirectory => CatalogConfig {
                catalog_item_name: b"BaseDirectory",
                out_folder_name: "base_directory",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::BaseTable => CatalogConfig {
                catalog_item_name: b"BaseTable",
                out_folder_name: "table_stubs",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::CalcsForCustomFunctions => CatalogConfig {
                catalog_item_name: b"CustomFunctionCalc",
                out_folder_name: "custom_function_calcs",
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "CustomFunctionCalc/CustomFunctionReference/@id",
            },
            Self::CustomFunctions => CatalogConfig {
                catalog_item_name: b"CustomFunction",
                out_folder_name: "custom_function_stubs",
                wrapped_in_object_list: true,
                uses_folders: true,
                id_path: "",
            },
            Self::CustomMenu => CatalogConfig {
                catalog_item_name: b"CustomMenu",
                out_folder_name: "custom_menus",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::CustomMenuSet => CatalogConfig {
                catalog_item_name: b"CustomMenuSet",
                out_folder_name: "custom_menu_sets",
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "",
            },
            Self::ExternalDataSource => CatalogConfig {
                catalog_item_name: b"ExternalDataSource",
                out_folder_name: "external_data_sources",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::ExtendedPrivileges => CatalogConfig {
                catalog_item_name: b"ExtendedPrivilege",
                out_folder_name: "extended_privileges",
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "",
            },
            Self::FieldsForTables => CatalogConfig {
                catalog_item_name: b"FieldCatalog",
                out_folder_name: "tables",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "FieldCatalog/BaseTableReference/@id",
            },
            Self::FileAccess => CatalogConfig {
                catalog_item_name: b"Authorization",
                out_folder_name: "file_access",
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "",
            },
            Self::Layout => CatalogConfig {
                catalog_item_name: b"Layout",
                out_folder_name: "layouts",
                wrapped_in_object_list: false,
                uses_folders: true,
                id_path: "Layout/LayoutReference/@id",
            },
            Self::Library => CatalogConfig {
                catalog_item_name: b"BinaryData",
                out_folder_name: "libraries",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "BinaryData/LibraryReference/@id",
            },
            Self::OptionsForValueLists => CatalogConfig {
                catalog_item_name: b"ValueList",
                out_folder_name: "value_lists",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "ValueList/ValueListReference/@id",
            },
            Self::PrivilegeSets => CatalogConfig {
                catalog_item_name: b"PrivilegeSet",
                out_folder_name: "privilege_sets",
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "",
            },
            Self::Relationship => CatalogConfig {
                catalog_item_name: b"Relationship",
                out_folder_name: "relationships",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::Script => CatalogConfig {
                catalog_item_name: b"Script",
                out_folder_name: "script_stubs",
                wrapped_in_object_list: false,
                uses_folders: true,
                id_path: "",
            },
            Self::StepsForScripts => CatalogConfig {
                catalog_item_name: b"Script",
                out_folder_name: "scripts",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "Script/ScriptReference/@id",
            },
            Self::TableOccurrence => CatalogConfig {
                catalog_item_name: b"TableOccurrence",
                out_folder_name: "table_occurrences",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::Theme => CatalogConfig {
                catalog_item_name: b"Theme",
                out_folder_name: "themes",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::ValueList => CatalogConfig {
                catalog_item_name: b"ValueList",
                out_folder_name: "value_lists",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
        }
    }
}

#[derive(Debug)]
pub struct CatalogConfig {
    pub catalog_item_name: &'static [u8],
    pub out_folder_name: &'static str,
    pub wrapped_in_object_list: bool,
    pub uses_folders: bool,
    pub id_path: &'static str,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputTree {
    #[value(
        name = "domain",
        help = "Use domain (e.g. catalog name) as the root folder"
    )]
    Domain,

    #[value(name = "db", help = "Use database name as the root folder (default)")]
    Db,
}

pub struct Flags {
    pub parse_all_lines: bool,
    pub lossless: bool,
    pub output_tree: OutputTree,
}
