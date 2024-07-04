#![warn(missing_docs)]

//! Provides a way to manager asynchronous tasks.
//!
//! See [`examples/counter`](https://github.com/Umatriz/egui-task-manager/blob/master/examples/counter/src/main.rs)
//! for more information.
//!
//! ## [`TaskManager`]
//!
//! The [`TaskManager`] is a core type that you must save in your app's state. Call [`TaskManager::add_collection`]
//! to register a new collection. And then call [`TaskManager::push_task`] when you want to add a new task.
//!
//! ## [`TasksCollection`] and [`CollectionData`]
//!
//! [`TasksCollection`] can be implemented for a type and then this type might be used as a type parameter
//! for several methods.
//!
//! [`CollectionData`] is a "dynamic" version of a type that implements [`TasksCollection`] although
//! not totally. It holds collection's name and executor. And in addition to this holds currently
//! running tasks and channel that receives data that tasks yield.
//!
//! ## [`Task`] and [`Caller`]
//!
//! [`Task`] has name and [`Caller`]. [`Caller`] can be either [`Standard`](Caller::Standard) or
//! [`Progressing`](Caller::Progressing).
//!
//! [`Caller::standard`] expects a future.
//!
//! [`Caller::progressing`] expects a closure with an argument of type [`TaskProgressShared`].
//! This type provides functionality for progress tracking and functions such [`TaskProgressShared::set_total`]
//! and [`TaskProgressShared::update`]. Which allows you to track your progress.
//!
//! For more information about progress see [`TaskProgressShared`].
//!
//! ## [`TaskExecutor`]
//!
//! A trait that determines task's execution.

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
/// egui_task_manager::setup!();
/// ```
pub mod setup {
    use std::time::Duration;

    use tokio::runtime::Runtime;

    /// Creates a new runtime.
    ///
    /// ```rust
    /// # use egui_task_manager::setup::runtime;
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
    /// # use egui_task_manager::setup::*;
    /// let rt = runtime();
    /// let _enter = rt.enter();
    ///
    /// spawn_runtime_thread(rt);
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
