use crate::arkham_errors::ArkhamError;
use crate::arkham_utility::{display_header_msg, get_user_input};
use crate::arkham_version::{log_version, read_version_info};
use std::path::Path;
use std::process::Command;

fn check_and_init_git(project_root: &str) -> Result<(), ArkhamError> {
    let git_dir = Path::new(project_root).join(".git");

    if !git_dir.exists() {
        println!("Git Repo not found in {}, initializing..", project_root);
        Command::new("git")
            .current_dir(project_root)
            .arg("init")
            .output()
            .map_err(|e| {
                ArkhamError::BackupError(format!("Failed to initialize git repo: {}", e))
            })?;
    }
    Ok(())
}

fn get_commit_id(project_root: &str) -> Result<String, ArkhamError> {
    let cid_output = Command::new("git")
        .current_dir(project_root)
        .args(&["rev-parse", "HEAD"])
        .output()
        .map_err(|e| ArkhamError::BackupError(format!("Failed to get commit ID: {}", e)))?;

    String::from_utf8(cid_output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|_| ArkhamError::BackupError("Invalid UTF-8 in commit ID".to_string()))
}

pub(crate) fn save_state() -> Result<(), ArkhamError> {
    display_header_msg("Saving Current Project State!");

    // read current version info
    let version_info = read_version_info()?;

    let current_version = match version_info {
        Some(ref version) => &version.current_version,
        None => {
            return Err(ArkhamError::BackupError(
                "Current Version not found!".to_string(),
            ))
        }
    };

    let project_root = match version_info {
        Some(ref version) => &version.project_root,
        None => {
            return Err(ArkhamError::BackupError(
                "Project Root not found!".to_string(),
            ))
        }
    };

    // Checking Git Repo
    check_and_init_git(project_root)?;

    // Staging and Commiting all files
    Command::new("git")
        .current_dir(project_root)
        .args(&["add", "."])
        .output()
        .map_err(|e| ArkhamError::BackupError(format!("Failed to stage files: {}", e)))?;

    // Log Message
    let built_by = get_user_input("Who's building it? : ")?;
    let commit_log = get_user_input(
        "Enter a commit message describing the changes [ DON'T ADD a ',' in the message ] : ",
    )?;

    // commiting
    Command::new("git")
        .current_dir(project_root)
        .args(&["commit", "-m", &format!("v_{}", current_version)])
        .output()
        .map_err(|e| ArkhamError::BackupError(format!("Failed to commit changes: {}", e)))?;

    // fetch commit id
    let commit_id = get_commit_id(project_root)?;
    log_version(&commit_log, &built_by, &commit_id)?;

    display_header_msg(&format!(
        "Successfully Saved state for version {}",
        current_version
    ));
    Ok(())
}


// Git Rollback
pub(crate) fn _restore_to_state(_version: &str) -> Result<bool, ArkhamError> {
    unimplemented!("To be Done!");
}
