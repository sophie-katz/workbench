#[derive(Debug, PartialEq)]
pub struct TaskArgs {
    pub target_task_path: Option<String>,
    pub task_args: Vec<String>,
}
