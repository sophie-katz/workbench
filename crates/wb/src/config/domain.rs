use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    pub tasks: Option<HashMap<String, Task>>,
    pub namespaces: Option<HashMap<String, Namespace>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum NamespaceOrTask {
    Namespace(Namespace),
    Task(Task),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Namespace {
    pub tasks: HashMap<String, Task>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Task {
    pub run: Run,
    pub shell: Option<Shell>,
    pub dependencies: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Run {
    String(String),
    Args(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Shell {
    Bool(bool),
    String(String),
}
