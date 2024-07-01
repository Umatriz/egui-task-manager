use std::{future::Future, pin::Pin};

use crate::task::{AnyTask, TaskData};

mod caller;
pub mod executors;
mod handle;
mod progress;

pub use caller::Caller;
pub use handle::*;
pub use progress::*;

type PinnedFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

/// Marker to determine if task is finished
pub(super) struct Finished;

const _: Option<Box<dyn TaskExecutor<()>>> = None;

pub trait TaskExecutor<P> {
    fn push(&mut self, task: AnyTask<P>);
    fn poll(&mut self, tasks: &[TaskData<P>]) -> ExecutionPoll<P>;
}

pub enum ExecutionPoll<P> {
    /// There's a task ready to be executed. [`TasksExecutor::execute`] must be called.
    Ready(AnyTask<P>),
    /// There's no tasks or you're waiting for others to finish.
    Pending,
}
