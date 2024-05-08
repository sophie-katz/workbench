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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Config {
    pub tasks: Option<HashMap<String, Task>>,
    pub namespaces: Option<HashMap<String, Namespace>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum NamespaceOrTask {
    Namespace(Namespace),
    Task(Task),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Namespace {
    pub tasks: HashMap<String, Task>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Example {
    pub description: Option<String>,
    pub run: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Task {
    pub run: Run,
    pub shell: Option<Shell>,
    pub dependencies: Option<Vec<String>>,
    pub inputs: Option<Files>,
    pub outputs: Option<Files>,
    pub usage: Option<String>,
    pub description: Option<String>,
    pub examples: Option<Vec<Example>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum Run {
    String(String),
    Args(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum Shell {
    Bool(bool),
    String(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum Files {
    List(Vec<String>),
    Object {
        include: Vec<String>,
        exclude: Vec<String>,
    },
}
