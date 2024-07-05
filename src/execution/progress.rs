use std::sync::{
    mpsc::{Receiver, SendError, Sender},
    Arc, OnceLock,
};

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
pub trait Progress: Send {
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

    /// Progress channel's sender.
    pub fn sender(&self) -> Sender<Box<dyn Progress>> {
        self.channel.sender()
    }

    /// Progress channel's receiver.
    pub fn receiver(&self) -> &Receiver<Box<dyn Progress>> {
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
/// to it's fields.
///
/// It can be accessed via [`Caller::Progressing`](crate::Caller).
/// ```rust
/// # use egui_task_manager::*;
///
/// // We need to define a progress type and implement `Progress` for it.
/// struct UnitProgress;
///
/// impl Progress for UnitProgress {
///     fn apply(&self, current: &mut u32) {
///         *current += 1;
///     }
/// }
///
/// Caller::progressing(|progress| async move {
///     // Set the total number of items or steps that needs to be completed
///     // eg. number of items in the downloading.
///     progress.set_total(5);
///     // Now we can use our type.
///     let _ = progress.update(UnitProgress);
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

    /// Progresses in the task.
    pub fn update<P: Progress + 'static>(
        &self,
        progress: P,
    ) -> Result<(), SendError<Box<dyn Progress>>> {
        self.sender.send(Box::new(progress))
    }

    /// Get the total value.
    pub fn total(&self) -> Option<u32> {
        self.total.get().copied()
    }

    /// Clones the sender and returns it.
    ///
    /// It is recommended to use [`update`](Self::update).
    pub fn sender(&self) -> Sender<Box<dyn Progress>> {
        self.sender.clone()
    }
}
