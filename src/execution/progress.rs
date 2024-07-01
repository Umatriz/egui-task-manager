use std::sync::{Arc, OnceLock};

use crossbeam::channel::{Receiver, Sender};

use crate::channel::Channel;

pub struct TaskProgress<P> {
    current: u32,
    total: Arc<OnceLock<u32>>,
    channel: Channel<P>,
}

impl<P> Default for TaskProgress<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> TaskProgress<P> {
    pub fn new() -> Self {
        Self {
            current: 0,
            total: Arc::new(OnceLock::new()),
            channel: Channel::new(),
        }
    }

    #[cfg(feature = "egui")]
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

    pub fn current_mut(&mut self) -> &mut u32 {
        &mut self.current
    }

    pub fn set_total(&self, total: u32) -> Result<(), u32> {
        self.total.set(total)
    }

    pub fn sender(&self) -> Sender<P> {
        self.channel.sender()
    }

    pub fn receiver(&self) -> Receiver<P> {
        self.channel.receiver()
    }

    pub fn share(&self) -> TaskProgressShared<P> {
        TaskProgressShared {
            total: self.total.clone(),
            progress_sender: self.sender(),
        }
    }
}

pub struct TaskProgressShared<P> {
    total: Arc<OnceLock<u32>>,
    progress_sender: Sender<P>,
}

impl<P> TaskProgressShared<P> {
    pub fn set_total(&self, total: u32) -> Result<(), u32> {
        self.total.set(total)
    }

    pub fn progress_sender(&self) -> Sender<P> {
        self.progress_sender.clone()
    }
}
