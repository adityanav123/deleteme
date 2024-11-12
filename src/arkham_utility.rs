use crate::arkham_constants::*;
use crate::arkham_errors::ArkhamError;
use figlet_rs::FIGfont;
use std::io;
use std::io::Write;

// Utility Methods
pub(crate) fn ascii_title_print() {
    let standard_font = FIGfont::standard().unwrap();
    let figure = standard_font
        .convert(&format!("{}", ARKHAM_ASCII_LOGO))
        .unwrap();
    println!("{}", figure);
}

pub(crate) fn insert_separator() {
    let sep_char = String::from(DISPLAY_HEADER_CHAR); // U+2324
    println!("{}", sep_char.repeat(40)); // line length is hardcoded
}

pub(crate) fn display_header_msg(message: &str) {
    let width = 80;
    let padding = 2;

    // Box drawing
    let top_left = "╔";
    let top_right = "╗";
    let bottom_left = "╚";
    let bottom_right = "╝";
    let horizontal = "═";
    let vertical = "║";

    println!("{}{}{}", top_left, horizontal.repeat(width - 2), top_right);

    for line in message.lines() {
        let content_width = width - 4; // -4 for borders and minimum spacing
        let padding_total = content_width - line.len();
        let padding_left = padding_total / 2;
        let padding_right = padding_total - padding_left;

        println!(
            "{}{}{}{}{}",
            vertical,
            " ".repeat(padding_left + 1),
            line,
            " ".repeat(padding_right + 1),
            vertical
        );
    }

    //  bottom border
    println!(
        "{}{}{}",
        bottom_left,
        horizontal.repeat(width - 2),
        bottom_right
    );
}

// ./arkham help
pub(crate) fn help_me() {
    ascii_title_print();
    display_header_msg(&format!(
        "Arkham Versioning Protocol: v{}\n- Author: {}",
        ARKHAM_VER, AUTHOR
    ));
    println!("Usage: arkham [OPTION] [ARGS]\n");
    println!("Options:");
    println!("   help                           ==> Display this general help information");
    println!("   help [TOPIC]                   ==> Display help for a specific topic");
    println!("   build [build-flags]            ==> Setup Arkham Versioning & Build the project");
    println!("   clean                          ==> Clean up the project");
    println!("   backup                         ==> Save the current project state via Git");
    println!("   archives                       ==> Display all version logs");
    println!("   archive-entry [VERSIONS...]    ==> Display logs for specific versions");
    println!("   app-status                     ==> Display Current App Information\n");
    println!("Topics for specific help:");
    println!("   version    ==> Information about versioning");
    println!("   git        ==> Information about Git integration\n");
    println!("Examples:");
    println!(" ./arkham help version");
    println!(" ./arkham backup");
    println!(" ./arkham archive-entry 3.51 3.52");
    println!(" ./arkham archives");
}

pub(crate) fn help_with(topic: &str) {
    match topic {
        "version" => {
            display_header_msg("Arkham Help: Versioning");
            println!("Arkham manages versioning for your project:");
            println!("- Versions are stored in .version.info");
            println!("- Format: MAJOR.MINOR (e.g., 3.51)");
            println!("- Version info is embedded in the executable, to check : run \'strings <executable> | tail -n 4\'");
            println!();
            println!("To update version after a successful build:");
            println!("- Choose 'yes | y' when prompted");
            println!("- Select MAJOR (1) or MINOR (0) update");
            println!();
            println!("To see all version logs:");
            println!(" ./arkham archives");
            println!("To see specific version logs:");
            println!(" ./arkham archive-entry 3.51");
        }
        "git" => {
            display_header_msg("Arkham Help: Git Integration");
            println!("Arkham provides basic Git integration:");
            println!("- backup:             Save current changes");
            println!("- restore [VERSION]:  Revert Project to previous version state [Not Implemented Yet]");
            println!();
            println!("Examples:");
            println!(" ./arkham backup");
            println!(" ./arkham restore 3.51");
        }
        _ => {
            println!("Unknown help topic: {}", topic);
            println!("Available topics: version, git");
        }
    }
}

pub(crate) fn debug_log(debug_message: &str) {
    println!("[DEBUG]: {}", debug_message);
}

pub(crate) fn not_implemented_yet(debug_message: &str) {
    display_header_msg(&format!("[not_implemented_yet]: {}", debug_message));
}

// Helper function to read user input
pub(crate) fn get_user_input(prompt: &str) -> Result<String, ArkhamError> {
    print!("{}", prompt);
    io::stdout().flush().map_err(|e| ArkhamError::IoError(e))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| ArkhamError::IoError(e))?;

    Ok(input.trim().to_string())
}
