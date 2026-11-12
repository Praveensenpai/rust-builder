pub const DEFAULT_CODESPACE_NAME: &str = "rust-builder";
pub const DEFAULT_IDLE_TIMEOUT: &str = "10m";
pub const REMOTE_PROJECTS_DIR: &str = "/workspaces/rust-builder/projects";
pub const STORAGE_CLEANUP_THRESHOLD_PERCENT: u8 = 80;

#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub codespace: String,
    pub target: Option<String>,
    pub release: bool,
    pub features: Option<String>,
    pub all_features: bool,
    pub no_default_features: bool,
    pub bin: Option<String>,
    pub package: Option<String>,
    pub clean: bool,
    pub force: bool,
    pub install: bool,
    pub keep_warm: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            codespace: DEFAULT_CODESPACE_NAME.to_string(),
            target: None,
            release: true,
            features: None,
            all_features: false,
            no_default_features: false,
            bin: None,
            package: None,
            clean: false,
            force: false,
            install: false,
            keep_warm: false,
        }
    }
}
