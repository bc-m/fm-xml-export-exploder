pub struct CommitRecordRequestsOptions;
#[allow(dead_code)]
impl CommitRecordRequestsOptions {
    pub const WITH_DIALOG: u32 = 128;
    pub const SKIP_DATA_ENTRY_VALIDATION: u32 = 256;
    pub const OVERRIDE_ESS_LOCKING_CONFLICTS: u32 = 512;
}

pub struct RefreshWindowOptions;
#[allow(dead_code)]
impl RefreshWindowOptions {
    pub const FLUSH_CACHED_JOIN_RESULTS: u32 = 256;
    pub const FLUSH_CACHED_EXTERNAL_DATA: u32 = 512;
}

pub struct GoToFieldOptions;
impl GoToFieldOptions {
    pub const SELECT_PERFORM: u32 = 4096;
}
