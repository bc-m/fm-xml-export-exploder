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
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "AccountsCatalog" => Some(Self::Accounts),
            "BaseDirectoryCatalog" => Some(Self::BaseDirectory),
            "BaseTableCatalog" => Some(Self::BaseTable),
            "CalcsForCustomFunctions" => Some(Self::CalcsForCustomFunctions),
            "CustomFunctionsCatalog" => Some(Self::CustomFunctions),
            "CustomMenuCatalog" => Some(Self::CustomMenu),
            "CustomMenuSetCatalog" => Some(Self::CustomMenuSet),
            "ExternalDataSourceCatalog" => Some(Self::ExternalDataSource),
            "ExtendedPrivilegesCatalog" => Some(Self::ExtendedPrivileges),
            "FieldsForTables" => Some(Self::FieldsForTables),
            "FileAccessCatalog" => Some(Self::FileAccess),
            "LayoutCatalog" => Some(Self::Layout),
            "LibraryCatalog" => Some(Self::Library),
            "OptionsForValueLists" => Some(Self::OptionsForValueLists),
            "PrivilegeSetsCatalog" => Some(Self::PrivilegeSets),
            "RelationshipCatalog" => Some(Self::Relationship),
            "ScriptCatalog" => Some(Self::Script),
            "StepsForScripts" => Some(Self::StepsForScripts),
            "TableOccurrenceCatalog" => Some(Self::TableOccurrence),
            "ThemeCatalog" => Some(Self::Theme),
            "ValueListCatalog" => Some(Self::ValueList),
            _ => None,
        }
    }

    pub const fn get_config(&self) -> CatalogConfig {
        match self {
            Self::Accounts => CatalogConfig {
                catalog_item_name: "Account",
                out_folder_name: "accounts",
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "Account/AccountReference/@id",
            },
            Self::BaseDirectory => CatalogConfig {
                catalog_item_name: "BaseDirectory",
                out_folder_name: "base_directory",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::BaseTable => CatalogConfig {
                catalog_item_name: "BaseTable",
                out_folder_name: "table_stubs",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::CalcsForCustomFunctions => CatalogConfig {
                catalog_item_name: "CustomFunctionCalc",
                out_folder_name: "custom_function_calcs",
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "CustomFunctionCalc/CustomFunctionReference/@id",
            },
            Self::CustomFunctions => CatalogConfig {
                catalog_item_name: "CustomFunction",
                out_folder_name: "custom_function_stubs",
                wrapped_in_object_list: true,
                uses_folders: true,
                id_path: "",
            },
            Self::CustomMenu => CatalogConfig {
                catalog_item_name: "CustomMenu",
                out_folder_name: "custom_menus",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::CustomMenuSet => CatalogConfig {
                catalog_item_name: "CustomMenuSet",
                out_folder_name: "custom_menu_sets",
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "",
            },
            Self::ExternalDataSource => CatalogConfig {
                catalog_item_name: "ExternalDataSource",
                out_folder_name: "external_data_sources",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::ExtendedPrivileges => CatalogConfig {
                catalog_item_name: "ExtendedPrivilege",
                out_folder_name: "extended_privileges",
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "",
            },
            Self::FieldsForTables => CatalogConfig {
                catalog_item_name: "FieldCatalog",
                out_folder_name: "tables",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "FieldCatalog/BaseTableReference/@id",
            },
            Self::FileAccess => CatalogConfig {
                catalog_item_name: "Authorization",
                out_folder_name: "file_access",
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "",
            },
            Self::Layout => CatalogConfig {
                catalog_item_name: "Layout",
                out_folder_name: "layouts",
                wrapped_in_object_list: false,
                uses_folders: true,
                id_path: "Layout/LayoutReference/@id",
            },
            Self::Library => CatalogConfig {
                catalog_item_name: "BinaryData",
                out_folder_name: "libraries",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "BinaryData/LibraryReference/@id",
            },
            Self::OptionsForValueLists => CatalogConfig {
                catalog_item_name: "ValueList",
                out_folder_name: "value_lists",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "ValueList/ValueListReference/@id",
            },
            Self::PrivilegeSets => CatalogConfig {
                catalog_item_name: "PrivilegeSet",
                out_folder_name: "privilege_sets",
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "",
            },
            Self::Relationship => CatalogConfig {
                catalog_item_name: "Relationship",
                out_folder_name: "relationships",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::Script => CatalogConfig {
                catalog_item_name: "Script",
                out_folder_name: "script_stubs",
                wrapped_in_object_list: false,
                uses_folders: true,
                id_path: "",
            },
            Self::StepsForScripts => CatalogConfig {
                catalog_item_name: "Script",
                out_folder_name: "scripts",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "Script/ScriptReference/@id",
            },
            Self::TableOccurrence => CatalogConfig {
                catalog_item_name: "TableOccurrence",
                out_folder_name: "table_occurrences",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::Theme => CatalogConfig {
                catalog_item_name: "Theme",
                out_folder_name: "themes",
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "",
            },
            Self::ValueList => CatalogConfig {
                catalog_item_name: "ValueList",
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
    pub catalog_item_name: &'static str,
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
    pub obfuscate_passwords: bool,
}
