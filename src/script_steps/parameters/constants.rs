pub mod commit_record_requests {
    #[allow(dead_code)]
    pub const WITH_DIALOG: u32 = 128;
    pub const SKIP_DATA_ENTRY_VALIDATION: u32 = 256;
    pub const OVERRIDE_ESS_LOCKING_CONFLICTS: u32 = 512;
}

pub mod refresh_window {
    pub const FLUSH_CACHED_JOIN_RESULTS: u32 = 256;
    pub const FLUSH_CACHED_EXTERNAL_DATA: u32 = 512;
}

pub mod go_to_field {
    pub const SELECT_PERFORM: u32 = 4096;
}
