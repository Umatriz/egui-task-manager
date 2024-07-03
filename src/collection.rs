use std::any::Any;

use crate::{
    any::IntoAny,
    channel::Channel,
    execution::{AnyHandle, Handle, TaskExecutor},
    task::{AnyTask, Task, TaskData},
};

/// Describes the collection of tasks.
///
/// ```rust
/// # use egui_task_manager::*;
///
/// struct SimpleCollection;
///
/// impl<'c, P> TasksCollection<'c, P> for SimpleCollection {
///     type Context = ();
///
///     type Target = String;
///
///     type Executor = executors::Linear<P>;
///
///     fn name() -> &'static str {
///         "Simple collection"
///     }
///
///     fn handle(context: Self::Context) -> Handle<'c, Self::Target> {
///         Handle::new(|result| println!("Received a value! {result}"))
///     }
/// }
/// ```
pub trait TasksCollection<'c> {
    /// Context that you can pass into the result handle.
    ///
    /// # Usage
    ///
    /// ```rust
    /// // This type will be available in the handle.
    /// type Context = &'c mut Vec<String>;
    /// ```
    type Context: 'c;

    /// The vale that tasks in this collection must return.
    type Target: Send + 'static;

    /// Executor that is used to control the execution process for this collection.
    type Executor: TaskExecutor + Default;

    /// Collections' name that will be displayed.
    fn name() -> &'static str;

    /// Handle that handles tasks' results. It can capture the context provided
    /// by the [`Context`](TasksCollection::Context).
    fn handle(context: Self::Context) -> Handle<'c, Self::Target>;
}

pub struct CollectionData {
    name: &'static str,
    channel: Channel<Box<dyn Any + Send>>,
    tasks: Vec<TaskData>,
    executor: Box<dyn TaskExecutor>,
}

impl CollectionData {
    #[cfg(feature = "egui")]
    pub fn ui(&self, ui: &mut egui::Ui) {
        ui.collapsing(self.name, |ui| {
            for task in &self.tasks {
                ui.group(|ui| task.ui(ui));
            }
        });
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub(super) fn from_collection<'c, C>() -> Self
    where
        C: TasksCollection<'c>,
        C::Executor: 'static,
    {
        Self {
            name: C::name(),
            channel: Channel::new(),
            tasks: Vec::new(),
            executor: Box::<C::Executor>::default(),
        }
    }

    fn execute(&mut self, task: AnyTask) {
        let sender = self.channel.sender();
        let task_data = task.execute(sender);
        self.push_task_data(task_data);
    }

    fn push_task_data(&mut self, task_data: TaskData) {
        self.tasks.push(task_data)
    }

    pub fn push_task<'c, C>(&mut self, task: Task<C::Target>)
    where
        C: TasksCollection<'c>,
        C::Target: Send,
    {
        self.executor.push(task.into_any());
    }

    pub fn handle_all(&mut self, result_handle: AnyHandle<'_>) {
        self.handle_execution();
        self.handle_progress();
        self.handle_results(result_handle);
        self.handle_deletion();
    }

    pub fn handle_results(&mut self, mut handle: AnyHandle<'_>) {
        if let Ok(value) = self.channel.receiver().try_recv() {
            handle.apply(value)
        }
    }

    pub fn handle_deletion(&mut self) {
        self.tasks.retain(|task| !task.is_finished())
    }

    pub fn handle_progress(&mut self) {
        for progress in self.tasks.iter_mut().filter_map(|task| task.progress_mut()) {
            if let Ok(data) = progress.receiver().try_recv() {
                data.apply(progress.current_mut())
            }
        }
    }

    pub fn handle_execution(&mut self) {
        use crate::execution::ExecutionPoll as E;
        while let E::Ready(task) = self.executor.poll(&self.tasks) {
            self.execute(task)
        }
    }
}
