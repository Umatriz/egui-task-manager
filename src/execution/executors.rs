//! Executors that can be used in the [`TasksCollection`](crate::TasksCollection).

use std::collections::VecDeque;

use crate::task::{AnyTask, TaskData};

use super::{ExecutionPoll, TaskExecutor};

/// Provides linear tasks execution.
///
/// Only one task might be executed at the time. The new task
/// will be executed as soon as the previous one is finished.
#[derive(Default)]
pub struct Linear {
    inner: VecDeque<AnyTask>,
}

impl TaskExecutor for Linear {
    fn push(&mut self, task: AnyTask) {
        self.inner.push_back(task)
    }

    fn poll(&mut self, tasks: &[TaskData]) -> ExecutionPoll {
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
#[derive(Default)]
pub struct Parallel {
    inner: VecDeque<AnyTask>,
}

impl TaskExecutor for Parallel {
    fn push(&mut self, task: AnyTask) {
        self.inner.push_back(task)
    }

    fn poll(&mut self, _tasks: &[TaskData]) -> ExecutionPoll {
        self.inner
            .pop_front()
            .map_or(ExecutionPoll::Pending, ExecutionPoll::Ready)
    }
}
