use crate::utils::calculate_hash;
use cached::proc_macro::cached;
use cached::SizedCache;
use glob::glob;
use rand::Rng;
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};
use tracing::{debug, error, warn};

#[cached(result = true, size = 100)]
pub fn get_compiler_args_from_xcodebuild(
    source_file_path: String,
    _hash: u64,
) -> Result<Vec<String>, XCodebuildError> {
    let path_to_xcodeproj = get_path_to_xcodeproj(source_file_path.to_string())?;

    let output = Command::new("xcodebuild")
        .arg("-project")
        .arg(path_to_xcodeproj)
        .arg("-showBuildSettingsForIndex")
        .arg("-alltargets")
        .arg("-json")
        .output()
        .map_err(|e| XCodebuildError::GenericError(e.into()))?;

    if output.stdout.is_empty() {
        return Err(XCodebuildError::CouldNotGetBuildSettingsFromXcodebuild(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let xcodebuild_output_obj: Value =
        serde_json::from_str(&stdout).map_err(|e| XCodebuildError::GenericError(e.into()))?;

    extract_compiler_args_from_xcodebuild_output(&xcodebuild_output_obj, &source_file_path)
}

pub fn extract_compiler_args_from_xcodebuild_output(
    xcodebuild_output: &Value,
    source_file_path: &str,
) -> Result<Vec<String>, XCodebuildError> {
    extract_compiler_args_from_xcodebuild_output_recursive(xcodebuild_output, source_file_path)?
        .ok_or(XCodebuildError::CouldNotExtractCompilerArgsForFile(
            source_file_path.to_string(),
            xcodebuild_output.clone(),
        ))
}

pub fn extract_compiler_args_from_xcodebuild_output_recursive(
    object: &Value,
    source_file_path: &str,
) -> Result<Option<Vec<String>>, XCodebuildError> {
    if let Value::Object(object) = object {
        for (key, value) in object.iter() {
            if key == source_file_path {
                if let Some(Value::Array(swift_ast_command_args)) =
                    value.get("swiftASTCommandArguments")
                {
                    return Ok(Some(
                        swift_ast_command_args
                            .into_iter()
                            .map(|arg| arg.to_string())
                            .collect::<Vec<String>>(),
                    ));
                } else {
                    return Err(XCodebuildError::CouldNotFindSwiftAstCommandArgsKey(
                        source_file_path.to_string(),
                        value.clone(),
                    ));
                }
            }
            if let Some(result) =
                extract_compiler_args_from_xcodebuild_output_recursive(value, source_file_path)?
            {
                return Ok(Some(result));
            }
        }
    }
    Ok(None)
}

// The project.pbxproj file contains the files in the project. It is one option for caching the compiler args.
pub fn get_hashed_pbxproj_modification_date_with_random_fallback(source_file_path: &str) -> u64 {
    match get_hashed_pbxproj_modification_date(source_file_path) {
        Err(e) => {
            warn!(
                ?e,
                "Unable to get hash for project.pbxproj modification date"
            );
            let mut rng = rand::thread_rng();
            rng.gen::<u64>()
        }
        Ok(res) => res,
    }
}

// The project.pbxproj file contains the files in the project. It is one option for caching the compiler args.
fn get_hashed_pbxproj_modification_date(source_file_path: &str) -> Result<u64, XCodebuildError> {
    let path_to_xcodeproj = get_path_to_xcodeproj(source_file_path.to_string())?;

    let path_to_pbxproj = Path::new(&path_to_xcodeproj).join("project.pbxproj");

    if !path_to_pbxproj.exists() {
        error!("No project.pbxproj found in .xcodeproj!");
        return Err(XCodebuildError::CouldNotFindPbxProj);
    }

    let metadata =
        fs::metadata(path_to_pbxproj).map_err(|e| XCodebuildError::GenericError(e.into()))?;
    let modified_time = metadata
        .modified()
        .map_err(|e| XCodebuildError::GenericError(e.into()))?;
    Ok(calculate_hash(&modified_time))
}

// Hash based on modification date of pbxproj file to cache based on file structure
// This kind-of also caches based on project, since different projects have no reason to have same modification date
#[cached(
    type = "SizedCache<u64, ()>",
    create = "{ SizedCache::with_size(1000) }",
    convert = r#"{  get_hashed_pbxproj_modification_date_with_random_fallback(&source_file_path) }"#
)]
pub fn log_list_of_module_names(source_file_path: String) {
    tauri::async_runtime::spawn(async move {
        match get_list_of_module_names(&source_file_path) {
            Ok(result) => debug!(?result, ?source_file_path, "List of module names"),
            Err(e) => warn!(?e, ?source_file_path, "Could not get list of module names"),
        };
    });
}

fn get_list_of_module_names(source_file_path: &str) -> Result<Vec<String>, XCodebuildError> {
    let path_to_xcodeproj = get_path_to_xcodeproj(source_file_path.to_string())?;

    // xcodebuild -alltargets -showBuildSettingsForIndex | sed -n '/-module-name/{n;p;}' | sort | uniq
    let xcodebuild_child = Command::new("xcodebuild")
        .arg("-project")
        .arg(path_to_xcodeproj)
        .arg("-showBuildSettingsForIndex")
        .arg("-alltargets")
        .arg("-json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| XCodebuildError::GenericError(e.into()))?;

    let sed_child = Command::new("sed")
        .arg("-n")
        .arg("/-module-name/{n;p;}")
        .stdin(Stdio::from(xcodebuild_child.stdout.ok_or(
            XCodebuildError::CouldNotGetListOfModuleNames("a".to_string()),
        )?))
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| XCodebuildError::GenericError(e.into()))?;

    let sort_child = Command::new("sort")
        .stdin(Stdio::from(sed_child.stdout.ok_or(
            XCodebuildError::CouldNotGetListOfModuleNames("b".to_string()),
        )?))
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| XCodebuildError::GenericError(e.into()))?;

    let uniq_child = Command::new("uniq")
        .stdin(Stdio::from(sort_child.stdout.ok_or(
            XCodebuildError::CouldNotGetListOfModuleNames("c".to_string()),
        )?))
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| XCodebuildError::GenericError(e.into()))?;

    let output = uniq_child
        .wait_with_output()
        .map_err(|e| XCodebuildError::GenericError(e.into()))?;

    if output.stdout.is_empty() {
        return Err(XCodebuildError::CouldNotGetListOfModuleNames(
            "d".to_string(),
        ));
    }

    let mut s = String::from_utf8_lossy(&output.stdout).to_string();

    // Remove whitespace and last comma; format as array
    s.retain(|c| !c.is_whitespace());

    let last_comma_index = s.rfind(",");
    if let Some(last_comma_index) = last_comma_index {
        s.replace_range(last_comma_index..last_comma_index + 1, "");
    }

    s = format!("[{}]", s);

    let result: Vec<String> =
        serde_json::from_str(&s).map_err(|e| XCodebuildError::GenericError(e.into()))?;

    Ok(result)
}

// TODO: Cache this according to whether we are still inside the folder? Or is it okay to recompute every time?
#[cached(result = true, time = 600)]
pub fn get_path_to_xcodeproj(file_path_str: String) -> Result<String, XCodebuildError> {
    let file_path = Path::new(&file_path_str);
    if !Path::new(file_path).exists() {
        return Err(XCodebuildError::FileNotExisting(file_path_str.to_string()));
    }
    for ancestor in file_path.ancestors() {
        let folder_str = ancestor.to_string_lossy();

        let pattern = format!("{}/*.xcodeproj", folder_str);
        if let Some(glob_result) = glob(&pattern)
            .map_err(|_| XCodebuildError::InvalidGlobPattern(pattern))?
            .next()
        {
            let xcodeproj_path = glob_result
                .map_err(|e| XCodebuildError::GenericError(e.into()))?
                .to_string_lossy()
                .to_string();

            return Ok(xcodeproj_path.to_string());
        }
    }
    return Err(XCodebuildError::CouldNotFindXcodeprojConfig(file_path_str));
}

#[derive(thiserror::Error, Debug)]
pub enum XCodebuildError {
    #[error(
        "Could not extract compiler args from xcodebuild: No swiftASTCommandArguments key found"
    )]
    CouldNotFindSwiftAstCommandArgsKey(String, Value),

    #[error("File does not exist: '{0}'")]
    FileNotExisting(String),

    #[error("Unable to get list of module names")]
    CouldNotGetListOfModuleNames(String),

    #[error("Could not find .xcodeproj config file")]
    CouldNotFindXcodeprojConfig(String),

    #[error("Getting build settings from xcodebuild failed")]
    CouldNotGetBuildSettingsFromXcodebuild(String),

    #[error("Could not find project.pbxproj")]
    CouldNotFindPbxProj,

    #[error(
        "Could not extract compiler args from xcodebuild: Invalid glob pattern for finding .xcodeproj config file"
    )]
    InvalidGlobPattern(String),

    #[error("Could not extract compiler args from xcodebuild: File key not found")]
    CouldNotExtractCompilerArgsForFile(String, Value),

    #[error("Something failed")]
    GenericError(#[source] anyhow::Error),
}
