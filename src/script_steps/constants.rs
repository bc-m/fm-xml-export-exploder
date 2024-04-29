use std::str::FromStr;
use strum_macros::EnumString;

#[derive(Debug, EnumString, PartialEq)]
pub enum ScriptStep {
    #[strum(
        serialize = "2",
        serialize = "3",
        serialize = "15",
        serialize = "52",
        serialize = "53",
        serialize = "54",
        serialize = "58",
        serialize = "100",
        serialize = "110",
        serialize = "162",
        serialize = "163",
        serialize = "170",
        serialize = "171",
        serialize = "173",
        serialize = "198",
        serialize = "204"
    )]
    Unknown,
    #[strum(serialize = "1")]
    PerformScript,
    #[strum(serialize = "4")]
    GoToPreviousField,
    #[strum(serialize = "5")]
    GoToNextField,
    #[strum(serialize = "6")]
    GoToLayout,
    #[strum(serialize = "7")]
    NewRecordRequest,
    #[strum(serialize = "8")]
    DuplicateRecordRequest,
    #[strum(serialize = "9")]
    DeleteRecordRequest,
    #[strum(serialize = "10")]
    DeleteAllRecords,
    #[strum(serialize = "16")]
    GoToRecordRequestPage,
    #[strum(serialize = "17")]
    GoToField,
    #[strum(serialize = "19")]
    CheckRecord,
    #[strum(serialize = "20")]
    CheckFoundSet,
    #[strum(serialize = "21")]
    UnsortRecords,
    #[strum(serialize = "22")]
    EnterFindMode,
    #[strum(serialize = "23")]
    ShowAllRecords,
    #[strum(serialize = "24")]
    ModifyLastFind,
    #[strum(serialize = "25")]
    OmitRecord,
    #[strum(serialize = "26")]
    OmitMultipleRecords,
    #[strum(serialize = "27")]
    ShowOmittedOnly,
    #[strum(serialize = "28")]
    PerformFind,
    #[strum(serialize = "32")]
    OpenHelp,
    #[strum(serialize = "38")]
    OpenManageDatabase,
    #[strum(serialize = "44")]
    ExitApplication,
    #[strum(serialize = "50")]
    SelectAll,
    #[strum(serialize = "55")]
    EnterBrowseMode,
    #[strum(serialize = "68")]
    IfStart,
    #[strum(serialize = "69")]
    Else,
    #[strum(serialize = "70")]
    IfEnd,
    #[strum(serialize = "71")]
    LoopStart,
    #[strum(serialize = "72")]
    ExitLoopIf,
    #[strum(serialize = "73")]
    LoopEnd,
    #[strum(serialize = "75")]
    CommitRecordRequests,
    #[strum(serialize = "76")]
    SetFieldData,
    #[strum(serialize = "79")]
    FixWindow,
    #[strum(serialize = "82")]
    NewFile,
    #[strum(serialize = "85")]
    AllowUserAbort,
    #[strum(serialize = "86")]
    SetErrorRecording,
    #[strum(serialize = "88")]
    OpenScriptWorkspace,
    #[strum(serialize = "89")]
    Comment,
    #[strum(serialize = "90")]
    HaltScript,
    #[strum(serialize = "91")]
    ReplaceFieldContents,
    #[strum(serialize = "93")]
    Beep,
    #[strum(serialize = "94")]
    SetUseSystemFormats,
    #[strum(serialize = "99")]
    GoToPortalRow,
    #[strum(serialize = "101")]
    CopyRecordRequest,
    #[strum(serialize = "102")]
    FlushCacheToDisk,
    #[strum(serialize = "103")]
    ExitScript,
    #[strum(serialize = "105")]
    OpenSettings,
    #[strum(serialize = "106")]
    CorrectWord,
    #[strum(serialize = "107")]
    SpellingOptions,
    #[strum(serialize = "108")]
    SelectDictionaries,
    #[strum(serialize = "109")]
    EditUserDictionary,
    #[strum(serialize = "112")]
    OpenManageValueLists,
    #[strum(serialize = "113")]
    OpenSharing,
    #[strum(serialize = "114")]
    OpenFileOptions,
    #[strum(serialize = "115")]
    AllowFormattingBar,
    #[strum(serialize = "118")]
    OpenHosts,
    #[strum(serialize = "121")]
    CloseWindow,
    #[strum(serialize = "122")]
    NewWindow,
    #[strum(serialize = "125")]
    IfElse,
    #[strum(serialize = "126")]
    ConstrainFoundSet,
    #[strum(serialize = "127")]
    ExtendFoundSet,
    #[strum(serialize = "129")]
    OpenFindReplace,
    #[strum(serialize = "140")]
    OpenManageDataSources,
    #[strum(serialize = "141")]
    SetVariable,
    #[strum(serialize = "145")]
    GoToObject,
    #[strum(serialize = "149")]
    OpenEditSavedFinds,
    #[strum(serialize = "151")]
    OpenManageLayouts,
    #[strum(serialize = "156")]
    OpenManageContainers,
    #[strum(serialize = "165")]
    OpenManageThemes,
    #[strum(serialize = "167")]
    RefreshObject,
    #[strum(serialize = "169")]
    ClosePopover,
    #[strum(serialize = "172")]
    UploadToServer,
    #[strum(serialize = "183")]
    OpenMyApps,
}

pub fn id_to_script_step(id: &str) -> ScriptStep {
    ScriptStep::from_str(id).unwrap_or(ScriptStep::Unknown)
}
