// Copyright 2024 Sophie Katz
//
// This file is part of Workbench.
//
// Workbench is free software: you can redistribute it and/or modify it under the terms of the GNU
// General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// Workbench is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
// even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Workbench. If not,
// see <https://www.gnu.org/licenses/>.

use std::path::PathBuf;

use crate::{config::Files, error::WorkbenchError};

pub fn resolve_paths(files: &Files) -> Result<Vec<PathBuf>, WorkbenchError> {
    let include_patterns = resolve_include_patterns(files);

    let exclude_glob_patterns = resolve_exclude_patterns(files)
        .into_iter()
        .map(|pattern| glob::Pattern::new(pattern.as_str()))
        .collect::<Result<Vec<glob::Pattern>, glob::PatternError>>()?;

    let mut result = Vec::new();

    for include_pattern in include_patterns {
        for path in glob::glob(include_pattern.as_str())? {
            let path = path?;

            if !exclude_glob_patterns
                .iter()
                .any(|exclude_glob_pattern| exclude_glob_pattern.matches_path(path.as_path()))
            {
                result.push(path);
            }
        }
    }

    Ok(result)
}

fn resolve_include_patterns(files: &Files) -> Vec<String> {
    match files {
        Files::List(patterns) => patterns
            .iter()
            .filter(|pattern| !pattern.starts_with('!'))
            .cloned()
            .collect(),
        Files::Object { include, .. } => include.clone(),
    }
}

fn resolve_exclude_patterns(files: &Files) -> Vec<String> {
    match files {
        Files::List(patterns) => patterns
            .iter()
            .filter(|pattern| pattern.starts_with('!'))
            .cloned()
            .map(|mut pattern| {
                pattern.remove(0);
                pattern
            })
            .collect(),
        Files::Object { exclude, .. } => exclude.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_files_list_empty() {
        let files = Files::List(Vec::new());

        let paths = resolve_paths(&files).unwrap();

        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_resolve_files_one_include_no_files() {
        let files = Files::List(Vec::new());

        let paths = resolve_paths(&files).unwrap();

        assert_eq!(paths.len(), 0);
    }
}
