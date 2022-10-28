use std::path::{Path, PathBuf};

use tauri::{
    api::process::{Command, CommandEvent},
    regex::Regex,
};
use tracing::warn;

use super::XcodeText;

#[derive(thiserror::Error, Debug)]
pub enum SwiftFormatError {
    #[error("Failed to get swift version {}", 0)]
    VersionParsingFailed(String),
    #[error("Formatter failed. {}", 0)]
    FormatFailed(String),
    #[error("Formatter could not run due to missing configuration.")]
    InsufficientContextForFormat,
    #[error("Something went wrong when executing this SwiftFormatter.")]
    GenericError(#[source] anyhow::Error),
}

// The optional file_path is used for finding .swiftformat config files
pub async fn format_code(
    input: &XcodeText,
    file_path: &Option<String>,
) -> Result<String, SwiftFormatError> {
    let mut command = tauri::api::process::Command::new_sidecar("swiftformat")
        .map_err(|err| SwiftFormatError::GenericError(err.into()))?;

    let args = get_swiftformat_args(file_path);

    command = command.args(args);

    let (mut rx, mut child) = command
        .spawn()
        .map_err(|err| SwiftFormatError::GenericError(err.into()))?;

    child
        .write(input.as_string().as_bytes())
        .expect("Failed to write to swiftformat");

    drop(child);

    let mut text_content = String::new();
    let mut error_content = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(line) => {
                text_content.push_str(&(line + "\n"));
            }
            CommandEvent::Stderr(line) => {
                error_content.push_str(&(line + "\n"));
            }
            _ => (),
        }
    }

    if !text_content.is_empty() {
        Ok(text_content)
    } else if !error_content.is_empty() {
        return Err(SwiftFormatError::FormatFailed(error_content.into()));
    } else {
        Err(SwiftFormatError::FormatFailed("Unknown error".into()))
    }
}

fn get_swiftformat_args(file_path: &Option<String>) -> Vec<String> {
    let mut args = vec!["--quiet"];

    let swift_version;
    match get_xcode_swift_version() {
        Ok(version) => {
            swift_version = version.to_owned();
            args.push("--swiftversion");
            args.push(swift_version.as_str());
        }
        Err(err) => {
            warn!(?err, "Failed to parse swift version");
        }
    }

    if let (Some(file_path), true) = (file_path, swiftformat_config_file_exists(file_path)) {
        // Use config from .swiftformat file if it exists
        args.push("--stdinpath");
        args.push(&file_path);
    } else {
        let mut default_config = vec![
          "--maxwidth",
          "100",
          "--wraparguments",
          "before-first",
          "--wrapparameters",
          "before-first",
          "--wrapcollections",
          "before-first",
          "--indent",
          "4",
          "--semicolons",
          "never",
          "--markcategories",
          "false",
          "--enable",
          "isEmpty,blankLineAfterImports,blankLinesBetweenImports,organizeDeclarations,preferDouble,sortedSwitchCases,wrapEnumCases,wrapSwitchCases",
      ];
        args.append(&mut default_config);
    }
    args.iter().map(|&s| s.into()).collect()
}

fn swiftformat_config_file_exists(path: &Option<String>) -> bool {
    if let Some(starting_directory) = path {
        let mut path: PathBuf = starting_directory.into();
        let file = Path::new(".swiftformat");

        loop {
            path.push(file);

            if path.is_file() {
                return true;
            }

            if !(path.pop() && path.pop()) {
                // remove file && remove parent
                return false;
            }
        }
    }
    false
}

fn get_xcode_swift_version() -> Result<String, SwiftFormatError> {
    let text_content = Command::new("xcrun")
        .args(["swift", "--version"])
        .output()
        .map_err(|e| SwiftFormatError::VersionParsingFailed(e.to_string()))?;

    if text_content.stdout.is_empty() {
        return Err(SwiftFormatError::VersionParsingFailed(text_content.stderr));
    } else {
        let version = text_content.stdout.split_whitespace().nth(3).ok_or(
            SwiftFormatError::VersionParsingFailed("Failed to find version".to_string()),
        )?;
        if !check_version_format(version) {
            return Err(SwiftFormatError::VersionParsingFailed(
                "Version is not correct format".to_string(),
            ));
        }
        Ok(version.to_string())
    }
}

fn check_version_format(version: &str) -> bool {
    let re = Regex::new(r"^\d(\.\d)+$").unwrap();
    re.is_match(version)
}

#[cfg(test)]
mod tests {

    mod check_version_format {
        use super::super::*;

        #[test]
        fn good_format() {
            assert!(check_version_format("5.3"));
            assert!(check_version_format("5.3.1"));
        }

        #[test]
        fn bad_format() {
            assert!(!check_version_format("(swiftlang-5.7.0.127.4"));
            assert!(!check_version_format("(a.1"));
            assert!(!check_version_format("(.b"));
            assert!(!check_version_format("(.1"));
            assert!(!check_version_format("(1"));
            assert!(!check_version_format("(1."));
        }
    }

    mod get_xcode_swift_version {

        use super::super::*;

        #[test]
        fn good_version() {
            let version = get_xcode_swift_version();
            assert!(version.is_ok());
        }
    }
}
