use std::{
    any::Any,
    sync::{Arc, OnceLock},
};

use crossbeam::channel::Sender;

use crate::{
    any::{HigherKinded, IntoAny},
    execution::{Caller, Finished, TaskProgress},
    spawning::TaskHandle,
};

/// Task that has `Box<dyn Any + Send>` as a return type.
pub type AnyTask<P> = Task<Box<dyn Any + Send>, P>;

/// A task.
pub struct Task<R, P> {
    name: String,
    is_finished: Arc<OnceLock<Finished>>,
    inner: Caller<R, P>,
}

impl<R: 'static + Send, P> Task<R, P> {
    /// Creates a new task using provided name and [`Caller`](crate::Caller).
    pub fn new(name: impl Into<String>, caller: Caller<R, P>) -> Self {
        Self {
            name: name.into(),
            is_finished: Arc::new(OnceLock::new()),
            inner: caller,
        }
    }

    /// Executes the task using provided `Sender` to send the result.
    pub(crate) fn execute(self, channel: Sender<R>) -> TaskData<P> {
        let (fut, progress) = match self.inner {
            Caller::Standard(fut) => (fut, None),
            Caller::Progressing(fun) => {
                let task_progress = TaskProgress::new();
                let fut = (fun)(task_progress.share());

                (fut, Some(task_progress))
            }
        };

        let is_finished = self.is_finished.clone();

        let handle = TaskHandle::from(async move {
            let value = fut.await;
            let _ = channel.send(value);
            let _ = is_finished.set(Finished);
        });

        TaskData {
            name: self.name,
            handle,
            is_finished: self.is_finished.clone(),
            progress,
        }
    }
}

impl<T, P> HigherKinded for Task<T, P> {
    type T<A> = Task<A, P>;
}

impl<T, P> IntoAny for Task<T, P>
where
    T: Send + 'static,
    P: 'static,
{
    fn into_any(self) -> Self::T<Box<dyn Any + Send>> {
        Task {
            name: self.name,
            is_finished: self.is_finished,
            inner: self.inner.into_any(),
        }
    }
}

/// The data of a task that is currently running.
pub struct TaskData<P> {
    name: String,
    handle: TaskHandle,
    is_finished: Arc<OnceLock<Finished>>,
    progress: Option<TaskProgress<P>>,
}

impl<P> TaskData<P> {
    #[cfg(feature = "egui")]
    /// Draws a simple ui.
    pub fn ui(&self, ui: &mut egui::Ui) {
        ui.label(self.name.as_str());
        match self.progress.as_ref() {
            Some(progress) => progress.ui(ui),
            None => {
                ui.spinner();
            }
        }

        let button = ui.button("Cancel");
        let popup_id = egui::Id::new("confirm_task_cancellation_popup_id");

        if button.clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        }

        egui::popup::popup_below_widget(ui, popup_id, &button, |ui| {
            ui.label("Are you sure you want to cancel the task?");
            ui.horizontal(|ui| {
                if ui.button("Yes").clicked() {
                    self.handle.abort();
                    let _ = self.is_finished.set(Finished);
                };
                if ui.button("No").clicked() {};
            });
        });
    }

    /// Tasks' name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Tasks' handle.
    ///
    /// Can be used to abort the task execution.
    pub fn handle(&self) -> &TaskHandle {
        &self.handle
    }

    /// Checks if the task finished or not.
    pub fn is_finished(&self) -> bool {
        self.is_finished.get().is_some()
    }

    /// Returns a reference to the [`TaskProgress`](crate::TaskProgress) of the current task if exists.
    pub fn progress(&self) -> Option<&TaskProgress<P>> {
        self.progress.as_ref()
    }

    /// Returns a mutable reference to the [`TaskProgress`](crate::TaskProgress) of the current task if exists.
    pub fn progress_mut(&mut self) -> Option<&mut TaskProgress<P>> {
        self.progress.as_mut()
    }
}
