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

use crate::config::Shell;

const DEFAULT_SHELL: &str = "/bin/sh";

pub fn resolve(shell: &Shell) -> Option<String> {
    match shell {
        Shell::Bool(true) => Some(DEFAULT_SHELL.to_owned()),
        Shell::Bool(false) => None,
        Shell::String(ref shell) => Some(shell.clone()),
    }
}
