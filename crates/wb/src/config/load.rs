use lazy_static::lazy_static;
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;

use super::Config;

lazy_static! {
    static ref CONFIG_FILENAMES: Vec<&'static str> = vec![
        "workbench.yaml",
        "workbench.yml",
        ".workbench.yaml",
        ".workbench.yml"
    ];
}

#[derive(Error, Debug)]
pub enum LoadingError {
    #[error("error while reading config file: {0}")]
    Io(#[from] io::Error),

    #[error("error while parsing config file: {0}")]
    Serde(#[from] serde_yaml::Error),
}

pub fn load(path: &Path) -> Result<Config, LoadingError> {
    let config_file = File::open(path)?;
    let config: Config = serde_yaml::from_reader(config_file)?;

    Ok(config)
}

pub fn resolve_path(
    starting_directory_path: &Path,
    filename_override: Option<&str>,
) -> Option<PathBuf> {
    let mut current_directory_path = starting_directory_path;

    loop {
        if let Some(config_file) =
            find_config_file_in_directory(current_directory_path, filename_override)
        {
            return Some(config_file);
        }

        if let Some(parent_directory_path) = current_directory_path.parent() {
            if parent_directory_path == current_directory_path {
                break;
            }

            current_directory_path = parent_directory_path;
        } else {
            break;
        }
    }

    None
}

fn find_config_file_in_directory(
    directory_path: &Path,
    filename_override: Option<&str>,
) -> Option<PathBuf> {
    if let Some(filename_override) = filename_override {
        let test_path = directory_path.join(filename_override);
        if is_file_or_symlink_to_file(&test_path) {
            Some(test_path.clone())
        } else {
            None
        }
    } else {
        for filename in CONFIG_FILENAMES.iter() {
            let path = directory_path.join(filename);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }
}

fn is_file_or_symlink_to_file(path: &Path) -> bool {
    path.is_file()
        || path
            .symlink_metadata()
            .map(|m| m.file_type().is_file())
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_is_file_or_symlink_to_file_file() {
        let temp_dir = tempfile::tempdir().unwrap();

        let file_path = temp_dir.path().join("file");

        File::create(&file_path).unwrap();

        assert!(is_file_or_symlink_to_file(&file_path));
    }

    #[test]
    fn test_is_file_or_symlink_to_file_directory() {
        let temp_dir = tempfile::tempdir().unwrap();

        assert!(!is_file_or_symlink_to_file(temp_dir.path()));
    }

    #[test]
    fn test_is_file_or_symlink_to_file_non_existent() {
        assert!(!is_file_or_symlink_to_file(&PathBuf::from(
            "this/file/does/not/exist"
        )));
    }

    #[test]
    fn test_is_file_or_symlink_to_file_symlink_to_file() {
        let temp_dir = tempfile::tempdir().unwrap();

        let file_path = temp_dir.path().join("file");

        let symlink_path = temp_dir.path().join("symlink");

        File::create(&file_path).unwrap();

        std::os::unix::fs::symlink(&file_path, &symlink_path).unwrap();

        assert!(is_file_or_symlink_to_file(&symlink_path));
    }

    #[test]
    fn test_is_file_or_symlink_to_file_symlink_to_directory() {
        let temp_dir = tempfile::tempdir().unwrap();

        let dir_path = temp_dir.path().join("dir");

        let symlink_path = temp_dir.path().join("symlink");

        fs::create_dir(&dir_path).unwrap();

        std::os::unix::fs::symlink(&dir_path, &symlink_path).unwrap();

        assert!(!is_file_or_symlink_to_file(&symlink_path));
    }

    #[test]
    fn test_is_file_or_symlink_to_file_broken_symlink() {
        let temp_dir = tempfile::tempdir().unwrap();

        let symlink_path = temp_dir.path().join("symlink");

        std::os::unix::fs::symlink(&PathBuf::from("this/file/does/not/exist"), &symlink_path)
            .unwrap();

        assert!(!is_file_or_symlink_to_file(&symlink_path));
    }

    #[test]
    fn test_find_config_file_in_directory_existent() {
        let temp_dir = tempfile::tempdir().unwrap();

        let file_path = temp_dir.path().join(".workbench.yaml");

        File::create(&file_path).unwrap();

        assert_eq!(
            find_config_file_in_directory(&temp_dir.path(), None),
            Some(file_path)
        );
    }

    #[test]
    fn test_find_config_file_in_directory_nonexistent() {
        let temp_dir = tempfile::tempdir().unwrap();

        let file_path = temp_dir.path().join(".workbench.yam");

        File::create(&file_path).unwrap();

        assert_eq!(find_config_file_in_directory(&temp_dir.path(), None), None,);
    }

    #[test]
    fn test_find_config_file_in_directory_existent_overridden() {
        let temp_dir = tempfile::tempdir().unwrap();

        let file_path = temp_dir.path().join("asdf.yaml");

        File::create(&file_path).unwrap();

        assert_eq!(
            find_config_file_in_directory(&temp_dir.path(), Some("asdf.yaml")),
            Some(file_path)
        );
    }

    #[test]
    fn test_find_config_file_in_directory_nonexistent_overridden() {
        let temp_dir = tempfile::tempdir().unwrap();

        let file_path = temp_dir.path().join("asdf.yam");

        File::create(&file_path).unwrap();

        assert_eq!(
            find_config_file_in_directory(&temp_dir.path(), Some("asdf.yaml")),
            None,
        );
    }

    #[test]
    fn test_resolve_path_in_current() {
        let temp_dir = tempfile::tempdir().unwrap();

        let file_path = temp_dir.path().join("workbench.yaml");

        File::create(&file_path).unwrap();

        assert_eq!(resolve_path(&temp_dir.path(), None), Some(file_path),);
    }

    #[test]
    fn test_resolve_path_in_parent() {
        let temp_dir = tempfile::tempdir().unwrap();

        let dir_path = temp_dir.path().join("dir");

        fs::create_dir(&dir_path).unwrap();

        let file_path = temp_dir.path().join("workbench.yaml");

        File::create(&file_path).unwrap();

        assert_eq!(resolve_path(dir_path.as_path(), None), Some(file_path),);
    }

    #[test]
    fn test_resolve_path_nonexistent() {
        let temp_dir = tempfile::tempdir().unwrap();

        let dir_path = temp_dir.path().join("dir");

        fs::create_dir(&dir_path).unwrap();

        assert_eq!(resolve_path(dir_path.as_path(), None), None);
    }
}
