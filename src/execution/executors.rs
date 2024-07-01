use std::collections::VecDeque;

use crate::task::{AnyTask, TaskData};

use super::{ExecutionPoll, TaskExecutor};

/// Provides linear tasks execution.
///
/// Only one task might be executed at the time. The new task
/// will be executed as soon as the previous one is finished.
pub struct Linear<P> {
    inner: VecDeque<AnyTask<P>>,
}

impl<P> Default for Linear<P> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<P> TaskExecutor<P> for Linear<P> {
    fn push(&mut self, task: AnyTask<P>) {
        self.inner.push_back(task)
    }

    fn poll(&mut self, tasks: &[TaskData<P>]) -> ExecutionPoll<P> {
        if !self.inner.is_empty() && !tasks.is_empty() {
            return ExecutionPoll::Pending;
        }

        self.inner
            .pop_front()
            .map_or(ExecutionPoll::Pending, ExecutionPoll::Ready)
    }
}

/// Provides parallel tasks execution.
///
/// Several tasks might be executed at the time. The new task
/// will be executed immediately.
pub struct Parallel<P> {
    inner: VecDeque<AnyTask<P>>,
}

impl<P> Default for Parallel<P> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<P> TaskExecutor<P> for Parallel<P> {
    fn push(&mut self, task: AnyTask<P>) {
        self.inner.push_back(task)
    }

    fn poll(&mut self, _tasks: &[TaskData<P>]) -> ExecutionPoll<P> {
        self.inner
            .pop_front()
            .map_or(ExecutionPoll::Pending, ExecutionPoll::Ready)
    }
}
