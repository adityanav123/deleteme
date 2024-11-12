extern crate tabled;
use tabled::Tabled;

// ARKHAM system files
pub(crate) const ARKHAM_VER: &str = "1.4";
pub(crate) const VERSION_INFO_FILE: &str = ".version.info";
pub(crate) const VERSION_LOGS_FILE: &str = ".version.log";
pub(crate) const LATEST_BUILD_LOGS: &str = "latest_build-log.LOG";
pub(crate) const ARKHAM_ASCII_LOGO: &str = "Arkham";
pub(crate) const AUTHOR: &str = "Aditya Navphule";
pub(crate) const AUTHOR_KNOX_ID: &str = "aditya.sn2";
pub(crate) const DISPLAY_HEADER_CHAR: char = '=';
pub(crate) const DEBUG_MODE: bool = false; // enabled when run as ./arkham -DEBUG build // work in progress

// ARKHAM GIT PROTECTED FILES
pub(crate) const ARKHAM_PROTECTED_FILES: [&str; 5] = [
    "arkham",
    VERSION_INFO_FILE,
    VERSION_LOGS_FILE,
    LATEST_BUILD_LOGS,
    "build_logs",
];

#[derive(Tabled)]
pub(crate) struct VersionRecord {
    #[tabled(rename = "Version Name\n(oldest to newest)")]
    pub(crate) version: String,
    #[tabled(rename = "Version Log")]
    pub(crate) log: String,
    #[tabled(rename = "Build Date")]
    pub(crate) date: String,
    #[tabled(rename = "Built By")]
    pub(crate) builder: String,
    #[tabled(rename = "Commit ID(Truncated)")]
    pub(crate) commit: String,
}
