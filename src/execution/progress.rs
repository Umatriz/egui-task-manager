use std::sync::{Arc, OnceLock};

use crossbeam::channel::{Receiver, Sender};

use crate::channel::Channel;

/// It is used to handle to execution progress.
///
/// # Usage
///
/// ```rust
/// # use egui_task_manager::Progress;
/// struct UnitProgress;
///
/// impl Progress for UnitProgress {
///     fn apply(&self, current: &mut u32) {
///         *current += 1;
///     }
/// }
/// ```
pub trait Progress {
    /// Apply the progress.
    fn apply(&self, current: &mut u32);
}

/// Execution progress of a task.
pub struct TaskProgress {
    current: u32,
    total: Arc<OnceLock<u32>>,
    channel: Channel<Box<dyn Progress>>,
}

impl Default for TaskProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskProgress {
    /// Returns an empty task progress.
    pub fn new() -> Self {
        Self {
            current: 0,
            total: Arc::new(OnceLock::new()),
            channel: Channel::new(),
        }
    }

    #[cfg(feature = "egui")]
    /// Draws a simple ui.
    ///
    /// # Panics
    ///
    /// It will panic in the debug builds if `self.total` is uninitialized.
    pub fn ui(&self, ui: &mut egui::Ui) {
        // Value must be initialized
        debug_assert!(self.total.get().is_some());
        if let Some(total) = self.total.get().copied() {
            ui.add(
                egui::ProgressBar::new(self.current as f32 / total as f32)
                    .text(format!("{}/{}", self.current, total)),
            );
        } else {
            ui.spinner();
        }
    }

    /// Mutable reference to the current progress.
    pub fn current_mut(&mut self) -> &mut u32 {
        &mut self.current
    }

    /// Sets the total value.
    pub fn set_total(&self, total: u32) -> Result<(), u32> {
        self.total.set(total)
    }

    /// Progress channels' sender.
    pub fn sender(&self) -> Sender<Box<dyn Progress>> {
        self.channel.sender()
    }

    /// Progress channels' receiver.
    pub fn receiver(&self) -> Receiver<Box<dyn Progress>> {
        self.channel.receiver()
    }

    /// Gives a shared access to the [`TaskProgress`].
    pub fn share(&self) -> TaskProgressShared {
        TaskProgressShared {
            total: self.total.clone(),
            sender: self.sender(),
        }
    }
}

/// Shared version of [`TaskProgress`] which does not provide any mutable access
/// to its' fields.
///
/// It can be accessed via [`Caller::Progressing`](crate::Caller).
/// ```rust
/// # use egui_task_manager::Caller;
/// Caller::progressing(|progress| async move {
///     // set the total number of items or steps that needs to be completed
///     // eg. number of items in the downloading.
///     progress.set_total(5);
///
///     let sender = progress.sender();
///
///     // now you can send your progress
///     // in this case the type for progress is `()`
///     sender.send(());
/// });
/// ```
pub struct TaskProgressShared {
    total: Arc<OnceLock<u32>>,
    sender: Sender<Box<dyn Progress>>,
}

impl TaskProgressShared {
    /// Sets the total value.
    pub fn set_total(&self, total: u32) -> Result<(), u32> {
        self.total.set(total)
    }

    /// Progresses in the task. It actually sends the `progress` using a channel.
    pub fn update<P: Progress + 'static>(&self, progress: P) {
        self.sender.send(Box::new(progress));
    }
}
