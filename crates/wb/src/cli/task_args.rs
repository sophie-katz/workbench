#[derive(Debug, PartialEq)]
pub struct TaskArgs {
    pub task_path: Option<String>,
    pub task_args: Vec<String>,
}
