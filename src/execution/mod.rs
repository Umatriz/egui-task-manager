use std::{future::Future, pin::Pin};

use crate::task::{AnyTask, TaskData};

mod caller;
pub mod executors;
mod handler;
mod progress;

pub use caller::Caller;
pub use handler::*;
pub use progress::*;

type PinnedFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

/// Marker to determine if the task is finished.
pub(super) struct Finished;

const _: Option<Box<dyn TaskExecutor>> = None;

/// A trait that determines task's execution.
///
/// For examples see implementation of the [`Linear`](executors::Linear) and [`Parallel`](executors::Parallel)
/// executors.
pub trait TaskExecutor {
    /// Push a new task to the executor.
    fn push(&mut self, task: AnyTask);

    /// Poll the executor to get it's state.
    ///
    /// - `tasks` is a slice of the currently running tasks.
    ///
    /// See [`ExecutionPoll`] for more information. And [`executors`](executors) module for examples.
    fn poll(&mut self, tasks: &[TaskData]) -> ExecutionPoll;

    /// Tasks that are currently waiting to be executed.
    fn iter_tasks(&self) -> Box<dyn Iterator<Item = &AnyTask> + '_>;
}

/// Indicates whether a task available to be executed or not.
pub enum ExecutionPoll {
    /// There's a task ready to be executed. It will cause [`TasksExecutor::execute`] to be called.
    Ready(AnyTask),
    /// You're not executing a new task. Nothing will happen.
    Pending,
}
