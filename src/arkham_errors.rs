use std::fmt::Formatter;
use std::{fmt, io};
use std::error::Error;

#[derive(Debug)] // For Easy Printing
pub(crate) enum ArkhamError {
    IoError(io::Error),
    InvalidVersion(String),
    NoVersionSpecified,
    VersionNotFound(String),
    MultipleVersionErrors(Vec<String>),
    CorruptVersionInfo(String),
    MissingVersionInfo,
    BuildError(String),
    BackupError(String),
}

// Custom error print format
impl fmt::Display for ArkhamError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(error) => write!(f, "IO Error: {}", error),
            Self::InvalidVersion(ver) => write!(
                f,
                "Invalid version format '{}'. Expected format: X.YY (e.g., 3.53, 2.05)",
                ver
            ),
            Self::NoVersionSpecified => write!(
                f,
                "No version specified. Usage: arkham archive-entry <version1> [version2] ..."
            ),
            Self::VersionNotFound(ver) => write!(f, "Version {} not found in logs", ver),
            Self::MultipleVersionErrors(errors) => {
                writeln!(f, "Multiple version errors! :")?;
                for err in errors {
                    writeln!(f, "  - {}", err)?;
                }
                Ok(())
            }
            Self::CorruptVersionInfo(corruption_detail) => {
                write!(f, "Corrupt version info: {}", corruption_detail)
            }
            Self::MissingVersionInfo => write!(f, "Version Information missing"),
            Self::BuildError(msg) => write!(f, "Build error: {}", msg),
            Self::BackupError(msg) => write!(f, "Error occurred during saving/restoring state!: {}", msg),
        }
    }
}

impl Error for ArkhamError {}
impl From<io::Error> for ArkhamError {
    fn from(error: io::Error) -> Self {
        ArkhamError::IoError(error)
    }
}

pub(crate) fn validate_version(version: &str) -> Result<(), ArkhamError> {
    // is of the format 4 (no decimal)
    if !version.contains('.') {
        return Err(ArkhamError::InvalidVersion(version.into()));
    }

    let parse_ver : Vec<&str> = version.split('.').collect();
    if parse_ver.len() != 2 {
        return Err(ArkhamError::InvalidVersion(version.into()));
    }

    // Major Version Part
    if parse_ver[0].parse::<u32>().is_err() {
        return Err(ArkhamError::InvalidVersion(version.into()));
    }

    // Minor Version Part
    match parse_ver[1].parse::<u32>() {
        Ok(minor) if minor <= 99 => Ok(()),
        _ => Err(ArkhamError::InvalidVersion(version.into())),
    }
}

