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

use std::collections::HashSet;

use lazy_static::lazy_static;

use clap::Parser;

/// Command line arguments
#[derive(Parser, Debug, PartialEq, Eq)]
#[command(version, about, long_about)]
pub struct WorkbenchArgs {
    /// The number of tasks to run in parallel
    #[arg(short, long, default_value_t = 0)]
    pub jobs: u32,

    /// Whether or not to enable verbose output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Path to the config file to load
    #[arg(short = 'f', long)]
    pub config: Option<String>,

    /// Whether or not to disable progress bars
    #[arg(long, default_value_t = false)]
    pub disable_progress: bool,

    /// Whether or not to disable console colors
    #[arg(long, default_value_t = false)]
    pub disable_color: bool,

    /// Whether or not to disable unicode characters in output
    #[arg(long, default_value_t = false)]
    pub disable_unicode: bool,
}

lazy_static! {
    pub static ref ARGS_WITH_VALUES: HashSet<String> = HashSet::from([
        "-j".to_owned(),
        "--jobs".to_owned(),
        "-f".to_owned(),
        "--config".to_owned()
    ]);
}
