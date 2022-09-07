use std::{
    fs::{create_dir_all, File},
    io::{Error, ErrorKind, Read, Write},
    path::PathBuf,
    process::Command,
};

type Result<T> = std::result::Result<T, Error>;

pub struct FileOnDisk {
    pub path: PathBuf,
    pub content: String,
}

impl FileOnDisk {
    pub fn new(path: PathBuf, content: String) -> Self {
        Self { path, content }
    }

    pub fn write(&self) -> Result<PathBuf> {
        write_text_to_file(&self.path, &self.content)
    }

    pub fn _read(&self) -> Result<String> {
        read_text_from_file(&self.path)
    }

    pub fn delete(&self) -> Result<()> {
        remove_file(&self.path)
    }
}

impl Drop for FileOnDisk {
    fn drop(&mut self) {
        let _ = self.delete();
    }
}

/// It creates a directory if it doesn't exist, creates a file in that directory, and writes the text content
/// to the file.
///
/// Arguments:
///
/// * `file_path`: The file path where the file will be saved.
/// * `content`: The content of the file to be saved.
///
/// Returns:
///
/// A Result<PathBuf> to the newly created file.
#[allow(dead_code)]
pub fn write_text_to_file(file_path: &PathBuf, content: &str) -> Result<PathBuf> {
    let mut dir_path = file_path.clone();
    if dir_path.extension().is_some() {
        dir_path.pop();
    } else {
        return Err(Error::new(ErrorKind::Other, "No extension found"));
    }
    create_dir_all(&dir_path)
        .and_then(|_| File::create(&file_path).map_err(Into::into))
        .and_then(|mut f| f.write_all(content.as_bytes()).map_err(Into::into))?;

    Ok(file_path.clone())
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

/// Returns the (canonicalized) path to the git root directory this path belongs to.
/// What it does:
/// * If the path is not in a git repository, it returns None.
/// * If the path does point to a location that does not exist, it returns None.
/// * If the path has a file extension at the end, it is popped off and the path is checked again.
///
/// Arguments:
///
/// * `dir`: The path to the directory to check. If it is a file path, it is converted to a directory path.
///
/// Returns:
///
/// An Option containing the `PathBuf` to the git root directory (canonicalized).
#[allow(dead_code)]
fn get_git_root_dir(dir: &PathBuf) -> Option<PathBuf> {
    // Check if the path provided contains a file name and extension at the end; if yes, remove it.
    let mut dir_path = dir.clone();
    if dir_path.extension().is_some() {
        dir_path.pop();
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(dir_path)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let root_dir = if let Ok(root_dir) = String::from_utf8(output.stdout) {
                root_dir
            } else {
                return None;
            };
            let root_dir = root_dir.trim_end_matches("\n").to_string();
            Some(PathBuf::from(root_dir))
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests_fs_utils {

    use std::fs;

    use rand::Rng;

    use super::*;

    #[test]
    fn test_write_text_to_file() {
        let mut rng = rand::thread_rng();
        let n1: u32 = rng.gen::<u32>();

        let content = format!("{}", n1);
        let file_name = "test.txt";

        let path_buf = std::env::temp_dir();

        // Check if error is returned if no extension exists
        let result_no_extension = write_text_to_file(&path_buf, &content);
        assert!(result_no_extension.is_err());

        // Check if file is written
        let file_path = std::env::temp_dir().join(file_name);
        let result_with_extension = write_text_to_file(&file_path, &content);
        assert!(result_with_extension.is_ok());
        assert!(file_path.exists());

        // Check if content stored in file is correct
        let content_loaded = read_text_from_file(&file_path).unwrap();
        assert_eq!(content, content_loaded);

        remove_file(&file_path).unwrap();
        assert_ne!(file_path.exists(), true);
    }

    struct FileSystemSetup {
        pub git_root_dir_name: PathBuf,
        pub test_folder_dir: PathBuf,
        pub test_file_path: PathBuf,
        pub test_dir_not_existing: PathBuf,
        pub test_dir_not_a_git_repo: PathBuf,
    }

    impl FileSystemSetup {
        pub fn new() -> Self {
            let mut rng = rand::thread_rng();
            let random_number: u32 = rng.gen::<u32>();
            let git_root_dir_name =
                std::env::temp_dir().join(format!("test_get_git_root_dir-{}", random_number));
            let test_folder_dir = git_root_dir_name.join("test");
            let test_file_path = test_folder_dir.join("test_file.txt");
            let test_dir_not_existing = test_folder_dir.join("test_folder_not_existing");

            // create an empty folder temp folder
            let _ = Command::new("mkdir")
                .arg(git_root_dir_name.clone())
                .output()
                .expect("failed to execute process");

            assert!(git_root_dir_name.exists());

            // create an empty git repository in a temporary directory
            let _ = Command::new("git")
                .arg("-C")
                .arg(git_root_dir_name.clone())
                .arg("init")
                .output()
                .expect("failed to execute process");

            // create a folder in the git repository
            let _ = Command::new("mkdir")
                .arg(test_folder_dir.clone())
                .output()
                .expect("failed to execute process");

            assert!(test_folder_dir.exists());

            // create a file in the git repository / test_folder
            let _ = Command::new("touch")
                .arg("-a")
                .arg(test_file_path.clone())
                .output()
                .expect("failed to execute process");

            Self {
                git_root_dir_name,
                test_folder_dir,
                test_file_path,
                test_dir_not_existing,
                test_dir_not_a_git_repo: std::env::temp_dir(),
            }
        }
    }

    impl Drop for FileSystemSetup {
        fn drop(&mut self) {
            // remove the git repository
            let _ = Command::new("rm")
                .arg("-rf")
                .arg(self.git_root_dir_name.clone())
                .output()
                .expect("failed to execute process");

            assert!(!self.git_root_dir_name.exists());
        }
    }

    #[test]
    #[ignore]
    fn test_get_git_root_dir() {
        let test_resources = FileSystemSetup::new();

        // Failing to get git root dir of non-existing directory
        // Yields the following message in the terminal: "fatal: cannot change to 'PATH_DIR': No such file or directory"
        assert_eq!(
            get_git_root_dir(&test_resources.test_dir_not_existing.clone()),
            None
        );

        // Failing to get git root dir of directory that exists but that is not a git repository
        // Yields the following message in the terminal: "fatal: not a git repository (or any of the parent directories): .git"
        assert_eq!(
            get_git_root_dir(&test_resources.test_dir_not_a_git_repo.clone()),
            None
        );

        // Successfully get git root dir of a git root dir path
        assert_eq!(
            get_git_root_dir(&test_resources.git_root_dir_name.clone()),
            Some(fs::canonicalize(test_resources.git_root_dir_name.clone()).unwrap())
        );

        // Successfully get git root dir of a directory that is in a git repository
        assert_eq!(
            get_git_root_dir(&test_resources.test_folder_dir.clone()),
            Some(fs::canonicalize(test_resources.git_root_dir_name.clone()).unwrap())
        );

        // Successfully get git root dir of a file that is in a git repository
        // Tests if the function correctly popps the filename + extension at the end of the provided path.
        assert_eq!(
            get_git_root_dir(&test_resources.test_file_path.clone()),
            Some(fs::canonicalize(test_resources.git_root_dir_name.clone()).unwrap())
        );
    }
}
