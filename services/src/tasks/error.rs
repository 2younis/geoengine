use snafu::Snafu;

use super::TaskId;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
#[snafu(context(suffix(false)))] // disables default `Snafu` suffix
pub enum TaskError {
    #[snafu(display("Task not found with id: {task_id}"))]
    TaskNotFound { task_id: TaskId },
    #[snafu(display("Task was aborted by the user: {task_id}"))]
    TaskAborted { task_id: TaskId },
}
