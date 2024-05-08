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

use crate::config::Config;

pub fn exec(config: &Config) -> bool {
    let mut task_names = get_task_names(config);

    task_names.sort();

    println!("available tasks:");

    for task_name in task_names {
        println!("  {task_name}");
    }

    true
}

fn get_task_names(config: &Config) -> Vec<String> {
    let mut task_names = vec![":ls".to_owned()];

    if let Some(tasks) = &config.tasks {
        for task_path in tasks.keys() {
            task_names.push(task_path.to_string());
        }
    }

    task_names
}
