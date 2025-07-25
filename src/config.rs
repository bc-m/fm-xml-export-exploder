use crate::OutputTree;

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
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
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

    pub fn get_config(&self) -> CatalogConfig {
        match self {
            Self::Accounts => CatalogConfig {
                catalog_item_name: b"Account".to_vec(),
                out_folder_name: "accounts".to_string(),
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "Account/AccountReference/@id".to_string(),
            },
            Self::BaseDirectory => CatalogConfig {
                catalog_item_name: b"BaseDirectory".to_vec(),
                out_folder_name: "base_directory".to_string(),
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "".to_string(),
            },
            Self::BaseTable => CatalogConfig {
                catalog_item_name: b"BaseTable".to_vec(),
                out_folder_name: "table_stubs".to_string(),
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "".to_string(),
            },
            Self::CalcsForCustomFunctions => CatalogConfig {
                catalog_item_name: b"CustomFunctionCalc".to_vec(),
                out_folder_name: "custom_function_calcs".to_string(),
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "CustomFunctionCalc/CustomFunctionReference/@id".to_string(),
            },
            Self::CustomFunctions => CatalogConfig {
                catalog_item_name: b"CustomFunction".to_vec(),
                out_folder_name: "custom_function_stubs".to_string(),
                wrapped_in_object_list: true,
                uses_folders: true,
                id_path: "".to_string(),
            },
            Self::CustomMenu => CatalogConfig {
                catalog_item_name: b"CustomMenu".to_vec(),
                out_folder_name: "custom_menus".to_string(),
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "".to_string(),
            },
            Self::CustomMenuSet => CatalogConfig {
                catalog_item_name: b"CustomMenuSet".to_vec(),
                out_folder_name: "custom_menu_sets".to_string(),
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "".to_string(),
            },
            Self::ExternalDataSource => CatalogConfig {
                catalog_item_name: b"ExternalDataSource".to_vec(),
                out_folder_name: "external_data_sources".to_string(),
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "".to_string(),
            },
            Self::ExtendedPrivileges => CatalogConfig {
                catalog_item_name: b"ExtendedPrivilege".to_vec(),
                out_folder_name: "extended_privileges".to_string(),
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "".to_string(),
            },
            Self::FieldsForTables => CatalogConfig {
                catalog_item_name: b"FieldCatalog".to_vec(),
                out_folder_name: "tables".to_string(),
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "FieldCatalog/BaseTableReference/@id".to_string(),
            },
            Self::FileAccess => CatalogConfig {
                catalog_item_name: b"Authorization".to_vec(),
                out_folder_name: "file_access".to_string(),
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "".to_string(),
            },
            Self::Layout => CatalogConfig {
                catalog_item_name: b"Layout".to_vec(),
                out_folder_name: "layouts".to_string(),
                wrapped_in_object_list: false,
                uses_folders: true,
                id_path: "Layout/LayoutReference/@id".to_string(),
            },
            Self::Library => CatalogConfig {
                catalog_item_name: b"BinaryData".to_vec(),
                out_folder_name: "libraries".to_string(),
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "BinaryData/LibraryReference/@id".to_string(),
            },
            Self::OptionsForValueLists => CatalogConfig {
                catalog_item_name: b"ValueList".to_vec(),
                out_folder_name: "value_lists".to_string(),
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "ValueList/ValueListReference/@id".to_string(),
            },
            Self::PrivilegeSets => CatalogConfig {
                catalog_item_name: b"PrivilegeSet".to_vec(),
                out_folder_name: "privilege_sets".to_string(),
                wrapped_in_object_list: true,
                uses_folders: false,
                id_path: "".to_string(),
            },
            Self::Relationship => CatalogConfig {
                catalog_item_name: b"Relationship".to_vec(),
                out_folder_name: "relationships".to_string(),
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "".to_string(),
            },
            Self::Script => CatalogConfig {
                catalog_item_name: b"Script".to_vec(),
                out_folder_name: "script_stubs".to_string(),
                wrapped_in_object_list: false,
                uses_folders: true,
                id_path: "".to_string(),
            },
            Self::StepsForScripts => CatalogConfig {
                catalog_item_name: b"Script".to_vec(),
                out_folder_name: "scripts".to_string(),
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "Script/ScriptReference/@id".to_string(),
            },
            Self::TableOccurrence => CatalogConfig {
                catalog_item_name: b"TableOccurrence".to_vec(),
                out_folder_name: "table_occurrences".to_string(),
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "".to_string(),
            },
            Self::Theme => CatalogConfig {
                catalog_item_name: b"Theme".to_vec(),
                out_folder_name: "themes".to_string(),
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "".to_string(),
            },
            Self::ValueList => CatalogConfig {
                catalog_item_name: b"ValueList".to_vec(),
                out_folder_name: "value_list_stubs".to_string(),
                wrapped_in_object_list: false,
                uses_folders: false,
                id_path: "".to_string(),
            },
        }
    }
}

#[derive(Debug)]
pub struct CatalogConfig {
    pub catalog_item_name: Vec<u8>,
    pub out_folder_name: String,
    pub wrapped_in_object_list: bool,
    pub uses_folders: bool,
    pub id_path: String,
}

pub struct Flags {
    pub parse_all_lines: bool,
    pub lossless: bool,
    pub output_tree: OutputTree,
}
