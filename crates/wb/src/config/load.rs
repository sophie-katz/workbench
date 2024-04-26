use lazy_static::lazy_static;
use std::{
    error::Error,
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
    let config_file = std::fs::File::open(path)?;
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
