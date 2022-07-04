use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::PathBuf,
};

type Result<T> = std::result::Result<T, std::io::Error>;

#[allow(dead_code)]
/// It creates a directory if it doesn't exist, creates a file in that directory, and writes the text content
/// to the file.
///
/// Arguments:
///
/// * `app_dir_path`: The directory where the file will be saved.
/// * `file_name`: The name of the file to be created.
/// * `content`: The content of the file to be saved.
///
/// Returns:
///
/// A Result<PathBuf> to the newly created file.
pub fn write_text_to_file(
    app_dir_path: PathBuf,
    file_name: &str,
    content: &str,
) -> Result<PathBuf> {
    let file_path = app_dir_path.join(file_name);
    create_dir_all(&app_dir_path)
        .and_then(|_| File::create(&file_path).map_err(Into::into))
        .and_then(|mut f| f.write_all(content.as_bytes()).map_err(Into::into))?;

    Ok(file_path)
}

/// It opens a file, reads its contents into a string, and returns the string
///
/// Arguments:
///
/// * `file_path`: The path to the file we want to read.
///
/// Returns:
///
/// A Result<String>
#[allow(dead_code)]
pub fn read_text_from_file(file_path: &PathBuf) -> Result<String> {
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// It removes a file from the filesystem
///
/// Arguments:
///
/// * `file_path`: The path to the file to be removed.
///
/// Returns:
///
/// A Result<()>
#[allow(dead_code)]
fn remove_file(file_path: &PathBuf) -> Result<()> {
    std::fs::remove_file(file_path)?;
    Ok(())
}

#[cfg(test)]
mod tests_fs_utils {

    use rand::Rng;

    use super::*;

    #[test]
    #[ignore]
    fn test_write_text_to_file() {
        let mut rng = rand::thread_rng();
        let n1: u32 = rng.gen::<u32>();

        let content = format!("{}", n1);
        let file_name = "test.txt";

        // Using the Tauri AppHandle one can get the app directory, by calling
        // app_handle.path_resolver().app_dir().unwrap()

        let path_buf = std::env::temp_dir();

        let _ = write_text_to_file(path_buf, file_name, &content);

        // Check if file exists
        let file_path = std::env::temp_dir().join(file_name);
        assert!(file_path.exists());

        // Check if content stored in file is correct
        let content_loaded = read_text_from_file(&file_path).unwrap();
        assert_eq!(content, content_loaded);

        remove_file(&file_path).unwrap();
        assert_ne!(file_path.exists(), true);
    }
}
