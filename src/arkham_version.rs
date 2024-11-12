/*
 STORES & MANIPULATES OVER VERSION INFO
*/
extern crate tabled;

use crate::arkham_constants::{VersionRecord, VERSION_INFO_FILE, VERSION_LOGS_FILE};
use crate::arkham_errors::*;
use crate::arkham_utility::{debug_log, display_header_msg, insert_separator};

use chrono::Local;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader, ErrorKind};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::{fs, io};
use tabled::{
    settings::{object::Segment, Alignment, Modify, Padding, Style, Width},
    Table,
};
#[derive(Debug)]
pub(crate) struct ProjectInfo {
    pub project_name: String,
    pub current_version: String,
    pub project_root: String,
}
impl ProjectInfo {
    // Constructor
    fn new(name: String, version: String, root: String) -> Self {
        ProjectInfo {
            project_name: name,
            current_version: version,
            project_root: root,
        }
    }
}

// Reading Version Info From File : .version.info
pub(crate) fn read_version_info() -> Result<Option<ProjectInfo>, ArkhamError> {
    // Check if File Exists
    if !Path::new(VERSION_INFO_FILE).exists() {
        return Ok(None);
    }

    // Reading the file
    let info_file = match File::open(VERSION_INFO_FILE) {
        Ok(file) => file,
        Err(er) => return Err(ArkhamError::from(er)),
    };

    // Start Reading by Reader
    let reader = BufReader::new(info_file);
    let mut project_name = String::new();
    let mut current_version = String::new();
    let mut project_root = String::new();

    // Read from Reader
    for line in reader.lines() {
        // if reading line threw error
        let line = match line {
            Ok(line) => line,
            Err(er) => return Err(ArkhamError::from(er)),
        };

        // Parse the line
        let parse_info: Vec<&str> = line.splitn(2, '=').collect(); // 2 splits with '=' delimiter // Eg;project_name='arkham'
        if parse_info.len() != 2 {
            return Err(ArkhamError::CorruptVersionInfo(
                "Invalid format in Version file".to_string(),
            ));
        }
        match parse_info[0] {
            "project_name" => project_name = String::from(parse_info[1]),
            "current_version" => current_version = String::from(parse_info[1]),
            "project_root" => project_root = String::from(parse_info[1]),
            _ => {}
        }
    }

    if project_name.is_empty() || current_version.is_empty() {
        Err(ArkhamError::CorruptVersionInfo(format!(
            "Invalid Version info please check: {}",
            VERSION_INFO_FILE
        )))
    } else {
        Ok(Some(ProjectInfo::new(
            project_name,
            current_version,
            project_root,
        )))
    }
}

// Write Version Info to File
pub(crate) fn write_version_info(
    project_name: &str,
    current_version: &str,
    project_root: &str,
) -> Result<(), ArkhamError> {
    // debug_log(&format!("Adding Versioning info to {}", VERSION_INFO_FILE));

    // Create File : will create if it doesn't exist & truncate if it does
    let mut info_file = File::create(VERSION_INFO_FILE)?;
    // Start writing to file
    writeln!(info_file, "project_name={}", project_name)?;
    writeln!(info_file, "current_version={}", current_version)?;
    writeln!(info_file, "project_root={}", project_root)?;
    Ok(())
}

// Update Version in File
pub(crate) fn update_version(
    current_version: &str,
    update_type: &str,
) -> Result<String, ArkhamError> {
    // parse and fetch current version
    let parse_info: Vec<&str> = current_version.splitn(2, ' ').collect();
    if parse_info.len() != 2 {
        return Err(ArkhamError::InvalidVersion(current_version.to_string()));
    }

    // Check major|minor version
    let major_ver: i32 = parse_info[0]
        .parse()
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid major version"))?;
    let minor_ver: i32 = parse_info[1]
        .parse()
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid minor version"))?;

    // Update to new version
    let new_version = match update_type {
        "major" | "1" => format!("{}.00", major_ver + 1),
        "minor" | "0" => format!("{}.{:02}", major_ver, minor_ver + 1),
        _ => current_version.to_string(), // if nothing then stay on the same version
    };
    Ok(new_version)
}

// Log Version Info
pub(crate) fn log_version(
    version_log: &str,
    built_by: &str,
    commit_id: &str,
) -> Result<(), ArkhamError> {
    let info = read_version_info()?.ok_or(ArkhamError::MissingVersionInfo)?;

    let build_date = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if !Path::new(VERSION_LOGS_FILE).exists() {
        let mut log_version_file = File::create(VERSION_LOGS_FILE)?;
        writeln!(
            log_version_file,
            "version_name,version_log,build_date,built_by,commit_id"
        )?; // Write Header
    }

    // Open file in append mode
    let mut log_file = fs::OpenOptions::new()
        .append(true)
        .open(VERSION_LOGS_FILE)?;

    // Write to the file : CSV format
    writeln!(
        log_file,
        "{},\"{}\",\"{}\",\"{}\",\"{}\"",
        info.current_version, version_log, build_date, built_by, commit_id
    )?;

    println!("Logged: Version {} by {}", info.current_version, built_by);
    Ok(())
}

pub(crate) fn show_specific_version_logs(versions: &[String]) -> Result<(), ArkhamError> {
    // Check if versions are specified
    if versions.is_empty() {
        return Err(ArkhamError::NoVersionSpecified);
    }

    // Pre-validate all specified versions
    let mut validation_err = Vec::new();
    for ver in versions {
        if let Err(e) = validate_version(ver) {
            validation_err.push(format!("{}", e));
        }
    }

    if !validation_err.is_empty() {
        return Err(ArkhamError::MultipleVersionErrors(validation_err));
    }

    if !Path::new(VERSION_LOGS_FILE).exists() {
        println!("No Version logs found!\n");
        return Ok(());
    }

    let log_file = File::open(VERSION_LOGS_FILE)?;
    let reader = BufReader::new(log_file);
    let mut records = Vec::new();
    let mut validation_errors = Vec::new();

    // Skip header line
    let mut lines = reader.lines();
    let _ = lines.next();

    // Process matching version lines
    for (line_num, line_result) in lines.enumerate() {
        let line = line_result?;

        // Parse CSV fields
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;

        for c in line.chars() {
            match c {
                '"' => in_quotes = !in_quotes,
                ',' if !in_quotes => {
                    fields.push(current_field.trim().trim_matches('"').to_string());
                    current_field.clear();
                }
                _ => current_field.push(c),
            }
        }
        fields.push(current_field.trim().trim_matches('"').to_string());

        // Only process if version matches
        if !versions.iter().any(|ver| ver == &fields[0]) {
            continue;
        }

        // Validate fields count
        if fields.len() != 5 {
            validation_errors.push(format!(
                "Corrupt version info at line {}: Expected 5 fields, found {}",
                line_num + 2,
                fields.len()
            ));
            continue;
        }

        // Create record
        records.push(VersionRecord {
            version: fields[0].clone(),
            log: fields[1].clone(),
            date: fields[2].clone(),
            builder: fields[3].clone(),
            commit: if fields[4].len() > 8 {
                format!("{}...", &fields[4][..8])
            } else {
                fields[4].clone()
            },
        });
    }

    // Handle validation errors
    if !validation_errors.is_empty() {
        return Err(ArkhamError::MultipleVersionErrors(validation_errors));
    }

    // Handle case where no matching versions found
    if records.is_empty() {
        let not_found: Vec<String> = versions
            .iter()
            .map(|v| format!("Version {} not found in logs", v))
            .collect();
        return Err(ArkhamError::MultipleVersionErrors(not_found));
    }

    // Create and style table
    let mut table = Table::new(records);

    let styled_table = table
        .with(Style::ascii())
        .with(Padding::new(1, 1, 0, 0))
        .with(
            Modify::new(Segment::all())
                .with(Width::wrap(20))
                .with(Alignment::center()),
        );

    // Print table with heading
    println!("\nVersion Log History:");
    println!("{}\n", styled_table);

    Ok(())
}

// Older Parser
// fn parse_version_row(line: &str) -> Vec<String> {
//     let mut fields = Vec::new();
//     let mut current_field = String::new();
//     let mut in_quotes = false;
//
//     for c in line.chars() {
//         match c {
//             '"' => in_quotes = !in_quotes,
//             ',' if !in_quotes => {
//                 fields.push(current_field.clone());
//                 current_field.clear();
//             }
//             _ => current_field.push(c),
//         }
//     }
//     fields.push(current_field);
//     fields
// }

// Fetch All the versions
pub(crate) fn show_version_logs() -> Result<(), ArkhamError> {
    if !Path::new(VERSION_LOGS_FILE).exists() {
        println!("No Version logs found!\n");
        return Ok(());
    }

    let log_file = File::open(VERSION_LOGS_FILE)?;
    let reader = BufReader::new(log_file);
    let mut records = Vec::new();
    let mut validation_errors = Vec::new();

    // Skip header line
    let mut lines = reader.lines();
    let _ = lines.next();

    // Process each line
    for (line_num, line_result) in lines.enumerate() {
        let line = line_result?;

        // Parse CSV fields
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;

        for c in line.chars() {
            match c {
                '"' => in_quotes = !in_quotes,
                ',' if !in_quotes => {
                    fields.push(current_field.trim().trim_matches('"').to_string());
                    current_field.clear();
                }
                _ => current_field.push(c),
            }
        }
        fields.push(current_field.trim().trim_matches('"').to_string());

        // Validate fields count
        if fields.len() != 5 {
            validation_errors.push(format!(
                "Corrupt version info at line {}: Expected 5 fields, found {}",
                line_num + 2, // +2 because we skipped header and 0-based index
                fields.len()
            ));
            continue;
        }

        // Validate version format
        if let Err(e) = validate_version(&fields[0]) {
            validation_errors.push(format!("Invalid version at line {}: {}", line_num + 2, e));
            continue;
        }

        // Create record
        records.push(VersionRecord {
            version: fields[0].clone(),
            log: fields[1].clone(),
            date: fields[2].clone(),
            builder: fields[3].clone(),
            commit: if fields[4].len() > 8 {
                format!("{}...", &fields[4][..8])
            } else {
                fields[4].clone()
            },
        });
    }

    // Handle validation errors
    if !validation_errors.is_empty() {
        return Err(ArkhamError::MultipleVersionErrors(validation_errors));
    }

    if records.is_empty() {
        println!("No Version logs found!\n");
        return Ok(());
    }

    // Create and style table
    let mut table = Table::new(records);

    let styled_table = table
        .with(Style::ascii())
        .with(Padding::new(1, 1, 0, 0))
        .with(
            Modify::new(Segment::all())
                .with(Width::wrap(20))
                .with(Alignment::center()),
        );

    // Print table with spacing
    println!("\nVersion History:");
    println!("{}\n", styled_table);

    Ok(())
}

// Fetching && Printing Current version info
pub(crate) fn current_version_info() -> Result<(), ArkhamError> {
    // reading the version file
    match read_version_info()? {
        Some(ver_info) => {
            println!("App/Executable Name:  {}", ver_info.project_name);
            println!("App Version:          {}", ver_info.current_version);
            println!("App Root Folder:      {}", ver_info.project_root);
            Ok(())
        }
        None => Err(ArkhamError::MissingVersionInfo),
    }
}

// Sliding Window Search
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle) // sliding window length : needle.len()
}

/*
    Perf update : instead of fs::copy() && fs::remove() using fs::rename() as low on resources
*/
fn update_executable_version(info: &ProjectInfo, version: &str) -> Result<(), ArkhamError> {
    let current_date = Local::now().format("%Y-%m-%d").to_string();

    // debug_log(&format!("Current date: {}", &current_date));

    // Create prev_builds directory if it doesn't exist
    let prev_builds_dir = "prev_builds";
    if !Path::new(prev_builds_dir).exists() {
        fs::create_dir(prev_builds_dir)?;
        debug_log("Created prev_builds directory");
    }

    if Path::new(&info.project_name).exists() {
        // read the existing executable
        let mut exec_content = fs::read(&info.project_name)?;

        // Search pattern: remove previous version
        let start_pattern = b"--VERSION_INFO_START--";

        // Find the first occurrence of version info
        if let Some(pos) = find_subsequence(&exec_content, start_pattern) {
            // debug_log("deleting prev version..");
            // Truncate at the start of first version info
            exec_content.truncate(pos);
        }

        // Create version info
        let version_info = format!(
            "\n--VERSION_INFO_START--\nVersion: {}\nBuild Date: {}\n--VERSION_INFO_END--\n",
            version, current_date
        )
        .into_bytes();

        // Append version info to executable
        exec_content.extend(version_info);

        // Debug : Print the last portion containing version info as a string
        // if let Ok(version_text) = String::from_utf8(
        //     exec_content
        //         .clone()
        //         .into_iter()
        //         .skip(exec_content.len() - 100)
        //         .collect(),
        // ) {
        //     debug_log("\nLast 100 bytes as text (including version info):");
        //     debug_log(&format!("{}", version_text));
        // }

        // New executable
        let versioned_name = format!("{}_v_{}", info.project_name, version);

        // Write the updated executable
        fs::write(&versioned_name, exec_content)?;

        // Update Permissions
        let mut perms = fs::metadata(&versioned_name)?.permissions();
        perms.set_mode(0o777); // read-write-exec
        fs::set_permissions(&versioned_name, perms)?;

        // Verify version info
        if !verify_version_info(&versioned_name, version)? {
            fs::remove_file(&versioned_name)?;
            return Err(ArkhamError::BuildError(
                "Failed to write version info to executable!".to_string(),
            ));
        }

        // remove old symlink
        if Path::new(&info.project_name).exists() {
            fs::remove_file(&info.project_name)?;
        }

        // create new symlink
        std::os::unix::fs::symlink(&versioned_name, &info.project_name)?;

        // Verify Symlink
        if !Path::new(&info.project_name).exists() {
            return Err(ArkhamError::BuildError(
                "Failed to create Symlink!".to_string(),
            ));
        }

        // move older versions to prev_builds/
        let current_dir = std::env::current_dir()?;
        for entry in fs::read_dir(&current_dir)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();

            // version exec move
            if file_name.starts_with(&format!("{}_v_", info.project_name))
                && file_name != versioned_name
            {
                let dest_path = format!("{}/{}", prev_builds_dir, file_name);
                fs::rename(file_name, dest_path)?;
            }
        }

        // Keep only last 10 builds in prev_builds
        cleanup_old_builds(prev_builds_dir, &info.project_name)?;

        // Verify final state
        if !Path::new(&versioned_name).exists() || !Path::new(&info.project_name).exists() {
            return Err(ArkhamError::BuildError(
                "Failed to verify final executable state".to_string(),
            ));
        }

        println!(
            "Current version = {} created, Older versions being moved to ./prev_builds/",
            versioned_name
        );
        println!(
            "Symlink '{}' points to '{}'",
            info.project_name, versioned_name
        );
    } else {
        return Err(ArkhamError::BuildError(format!(
            "Executable {} not found",
            info.project_name
        )));
    }

    Ok(())
}

// just for verification
fn verify_version_info(executable: &str, version: &str) -> Result<bool, ArkhamError> {
    let output = Command::new("strings").arg(executable).output()?;

    let content = String::from_utf8_lossy(&output.stdout);
    Ok(content.contains(&format!("Version: {}", version)))
}

fn cleanup_old_builds(prev_builds_dir: &str, base_name: &str) -> Result<(), ArkhamError> {
    let mut builds: Vec<(String, fs::Metadata)> = Vec::new();

    for entry in fs::read_dir(prev_builds_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_name.starts_with(&format!("{}_", base_name)) && !file_name.ends_with(".version") {
            builds.push((file_name, entry.metadata()?));
        }
    }

    // Sort = newest first
    builds.sort_by(|a, b| b.1.modified().unwrap().cmp(&a.1.modified().unwrap()));

    // (keep only 10 most recent)
    for (file_name, _) in builds.iter().skip(10) {
        let exec_path = format!("{}/{}", prev_builds_dir, file_name);
        let version_path = format!("{}/{}.version", prev_builds_dir, file_name);

        if Path::new(&exec_path).exists() {
            fs::remove_file(&exec_path)?;
        }
        if Path::new(&version_path).exists() {
            fs::remove_file(&version_path)?;
        }
    }

    Ok(())
}

// Build & Clean
pub(crate) fn build_project(args: &[String]) -> Result<bool, ArkhamError> {
    display_header_msg("Building Project!");

    // Make Command
    let mut make_cmd = Command::new("make");
    make_cmd.args(args); // pass extra args

    // make output
    let build_output = make_cmd.output()?;
    let mut success_build = true;

    // Display Output
    let stdout = String::from_utf8_lossy(&build_output.stdout);
    for line in stdout.lines() {
        if line.contains("error:")
            || line.contains("Error ")
            || (line.contains("make:***") && !line.contains("is up to date"))
        {
            println!();
            insert_separator();
            println!("{}", line);
            insert_separator();
            success_build = false;
        } else {
            println!("{}", line);
        }
    }

    Ok(success_build)
}

pub(crate) fn clean_project() -> Result<(), ArkhamError> {
    display_header_msg("Cleaning Project Files!");
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Clean CMD
    let clean_output = Command::new("make").arg("clean").output()?;

    // clean cmd output
    println!("{}", String::from_utf8_lossy(&clean_output.stdout));

    if !clean_output.status.success() {
        return Err(ArkhamError::BuildError("Make Clean failed!".to_string()));
    }

    Ok(())
}

// build & update
pub(crate) fn build_and_update(args: &[String]) -> Result<(), ArkhamError> {
    // Fetch / Create app version
    let info = match read_version_info()? {
        Some(info) => {
            println!("Found Existing Version(s) - fetching..");
            std::thread::sleep(std::time::Duration::from_secs(1));
            display_header_msg(&format!(
                "Project Name: {}\nCurrent Version: {}",
                info.project_name, info.current_version
            ));
            info
        }
        None => {
            // Take info from User
            println!("No Versioning Found! Please enter details manually: ");
            print!("Enter Executable name (Eg. CookieUFS): ");
            io::stdout().flush()?; // print! doesn't flush output buffer automatically
            let mut project_name = String::new();
            io::stdin().read_line(&mut project_name)?;

            print!("Enter the current version (Eg. 3.53): ");
            io::stdout().flush()?; // print! doesn't flush output buffer automatically
            let mut project_version = String::new();
            io::stdin().read_line(&mut project_version)?;

            // Creating New Project Info
            let info = ProjectInfo {
                project_name: project_name.trim().to_string(),
                current_version: project_version.trim().to_string(),
                project_root: std::env::current_dir()?.to_string_lossy().to_string(),
            };

            write_version_info(
                &info.project_name,
                &info.current_version,
                &info.project_root,
            )?;
            info
        }
    };

    if build_project(args)? {
        display_header_msg(&format!("{}: Got built successfully!", info.project_name));

        print!("Do you want to update the version? (yes [y] | no [n]) = ");
        io::stdout().flush()?;
        let mut update_choice = String::new();
        io::stdin().read_line(&mut update_choice)?;

        if update_choice.trim().eq_ignore_ascii_case("yes")
            || update_choice.trim().eq_ignore_ascii_case("y")
        {
            // Get update type
            print!("Is this a Major or Minor Update? (MAJOR [1] | MINOR [0]) = ");
            io::stdout().flush()?;

            let mut update_type = String::new();
            io::stdin().read_line(&mut update_type)?;

            // Calculate new version
            let new_version = match update_type.trim().to_lowercase().as_str() {
                "major" | "1" => {
                    let parts: Vec<&str> = info.current_version.split('.').collect();
                    if parts.len() != 2 {
                        return Err(ArkhamError::InvalidVersion(info.current_version));
                    }
                    let major: i32 = parts[0]
                        .parse()
                        .map_err(|_| ArkhamError::InvalidVersion(info.current_version.clone()))?;
                    format!("{}.00", major + 1)
                }
                "minor" | "0" => {
                    let parts: Vec<&str> = info.current_version.split('.').collect();
                    if parts.len() != 2 {
                        return Err(ArkhamError::InvalidVersion(info.current_version));
                    }
                    let major: i32 = parts[0]
                        .parse()
                        .map_err(|_| ArkhamError::InvalidVersion(info.current_version.clone()))?;
                    let minor: i32 = parts[1]
                        .parse()
                        .map_err(|_| ArkhamError::InvalidVersion(info.current_version.clone()))?;
                    format!("{}.{:02}", major, minor + 1)
                }
                _ => {
                    return Err(ArkhamError::InvalidVersion(
                        "Invalid update type".to_string(),
                    ))
                }
            };

            // Update version info
            write_version_info(&info.project_name, &new_version, &info.project_root)?;
            update_executable_version(&info, &new_version)?;

            display_header_msg(&format!(
                "Version Successfully Updated from {} to {}",
                info.current_version, new_version
            ));
        } else {
            display_header_msg(&format!("Version unchanged: {}", info.current_version));
            update_executable_version(&info, &info.current_version)?;
        }
    } else {
        display_header_msg("Build failed! Check the log file");
        return Err(ArkhamError::BuildError("Build failed".to_string()));
    }

    Ok(())
}
