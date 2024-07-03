#![warn(missing_docs)]

//! The crate

mod any;
mod channel;
mod collection;
mod execution;
mod manager;
mod spawning;
mod task;

pub use collection::*;
pub use execution::*;
pub use manager::*;
pub use task::*;

/// Provides several functions and a macro to setup the runtime.
///
/// Most of the time you just need to call the macro and it will do everything for you.
/// ```rust
/// egui_task_manager::setup!()
/// ```
pub mod setup {
    use std::time::Duration;

    use tokio::runtime::Runtime;

    /// Creates a new runtime.
    ///
    /// ```rust
    /// use egui_task_manager::setup::runtime;
    ///
    /// // Keep the guard
    /// let _enter = runtime().enter();
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if [`Runtime::new`] panics.
    pub fn runtime() -> Runtime {
        tokio::runtime::Runtime::new().expect("Unable to create Runtime")
    }

    /// Executes the runtime in its own thread
    ///
    /// ```rust
    /// use egui_task_manager::setup::*;
    ///
    /// let rt = runtime();
    /// let _enter = rt.enter();
    ///
    /// spawn_sleeping_thread(rt);
    ///
    /// ```
    pub fn spawn_runtime_thread(rt: Runtime) {
        std::thread::spawn(move || {
            rt.block_on(async { tokio::time::sleep(Duration::from_secs(3600)).await })
        });
    }

    #[macro_export]
    /// Creates a tokio runtime and executes it in its own thread.
    ///
    /// ```rust
    /// egui_task_manager::setup!();
    /// ```
    /// For more information see
    /// [`setup::runtime`](crate::setup::runtime) and
    /// [`setup::spawn_runtime_thread`](crate::setup::spawn_runtime_thread).
    macro_rules! setup {
        () => {
            let runtime = $crate::setup::runtime();
            let _enter = runtime.enter();

            $crate::setup::spawn_runtime_thread(runtime);
        };
    }
}
