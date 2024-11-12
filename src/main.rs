mod arkham_constants;
mod arkham_errors;
mod arkham_git;
mod arkham_utility;
mod arkham_version;

extern crate figlet_rs;
use crate::arkham_errors::ArkhamError;
use crate::arkham_git::*;
use crate::arkham_version::*;
use arkham_utility::*;
use std::env;

// -MAIN-
fn main() {
    // Fetch CLI args
    let args: Vec<String> = env::args().collect();

    let result: Result<(), ArkhamError> = match args.get(1).map(|s| s.as_str()) {
        Some("help") => match args.get(2) {
            Some(topic) => {
                help_with(topic);
                Ok(())
            }
            None => {
                help_me();
                Ok(())
            }
        },
        Some("build") => {
            let build_args: Vec<String> = args.iter().skip(2).cloned().collect(); // [cookieUFS build](ignore) V=1
            match build_and_update(&build_args) {
                Ok(_) => Ok(()),
                Err(e) => {
                    match e {
                        ArkhamError::BuildError(ref msg) => {
                            println!("Build Error: {}", msg);
                            println!("Check the build logs for more details.");
                        }
                        ArkhamError::InvalidVersion(ref ver) => {
                            println!("Invalid version format: {}", ver);
                            println!("Version should be in format X.YY (e.g., 3.54)");
                        }
                        _ => println!("Error during *Make*: {}", e),
                    }
                    Err(e)
                }
            }
        }
        Some("clean") => match clean_project() {
            Ok(_) => {
                display_header_msg("Project cleaned successfully!");
                Ok(())
            }
            Err(e) => {
                // println!("Clean failed: {}", e);
                Err(e)
            }
        },
        Some("backup") => match save_state() {
            Ok(_) => {
                // display_header_msg("Project state saved successfully!");
                Ok(())
            }
            Err(e) => {
                match e {
                    ArkhamError::BackupError(ref msg) => {
                        println!("Backup Error: {}", msg);
                        println!("Failed to save project state.");
                    }
                    ArkhamError::IoError(ref err) => {
                        println!("IO Error during backup: {}", err);
                    }
                    _ => println!("Unexpected error during backup: {}", e),
                }
                Err(e)
            }
        },
        Some("archives") => match show_version_logs() {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    ArkhamError::MultipleVersionErrors(ref errors) => {
                        println!("Error while processing versions:");
                        for error in errors {
                            println!("  - {}", error);
                        }
                    }
                    _ => println!("Error: {}", e),
                }
                Err(e)
            }
        },
        Some("archive-entry") => {
            // Fetch all versions from CLI
            let versions: Vec<String> = args.iter().skip(2).cloned().collect();

            match show_specific_version_logs(&versions) {
                Ok(_) => Ok(()),
                Err(e) => {
                    // Print Help if wrong command
                    match e {
                        ArkhamError::NoVersionSpecified => {
                            println!("Error: {}", e);
                            println!("Example Usage: ");
                            println!("  arkham archive-entry 3.53 1.54");
                        }
                        ArkhamError::MultipleVersionErrors(ref errors) => {
                            println!("Error while processing versions:");
                            for error in errors {
                                println!("  - {}", error);
                            }
                            println!("Example Usage: ");
                            println!("  arkham archive-entry 3.53 1.54");
                        }
                        _ => println!("Error: {}", e),
                    }
                    Err(e)
                }
            }
        }
        Some("app-status") => current_version_info(),
        _ => {
            help_me();
            Ok(())
        }
    };
    if let Err(_err) = result {
        display_header_msg("Error!");
        std::process::exit(1); // Exiting if Error Occurred, Don't want the app to panic
    }
}
