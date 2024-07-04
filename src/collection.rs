use std::any::Any;

use crate::{
    channel::Channel,
    execution::{AnyHandler, Handler, TaskExecutor},
    task::{AnyTask, TaskData},
};

/// Describes the collection of tasks.
///
/// ```rust
/// # use egui_task_manager::*;
///
/// struct SimpleCollection;
///
/// impl<'c> TasksCollection<'c> for SimpleCollection {
///     type Context = ();
///
///     type Target = String;
///
///     type Executor = executors::Linear;
///
///     fn name() -> &'static str {
///         "Simple collection"
///     }
///
///     fn handle(context: Self::Context) -> Handler<'c, Self::Target> {
///         Handler::new(|result| println!("Received a value! {result}"))
///     }
/// }
/// ```
pub trait TasksCollection<'c> {
    /// Context that you can pass into the result handle.
    type Context: 'c;

    /// The vale that tasks in this collection must return.
    type Target: Send + 'static;

    /// Executor that is used to control the execution process for this collection.
    type Executor: TaskExecutor + Default;

    /// Collection's name that will be displayed.
    fn name() -> &'static str;

    /// Handle that handles task's results. It can capture the context provided
    /// by the [`Context`](TasksCollection::Context).
    fn handle(context: Self::Context) -> Handler<'c, Self::Target>;
}

/// Collection holds the tasks in the queue and the data of currently executing ones.
///
/// It uses [`TaskExecutor`](crate::TaskExecutor) to determine when a new task should
/// start it's execution.
pub struct CollectionData {
    name: &'static str,
    channel: Channel<Box<dyn Any + Send>>,
    tasks: Vec<TaskData>,
    executor: Box<dyn TaskExecutor>,
}

impl CollectionData {
    #[cfg(feature = "egui")]
    /// Draws a simple ui.
    pub fn ui(&self, ui: &mut egui::Ui) {
        ui.collapsing(self.name, |ui| {
            for task in &self.tasks {
                ui.group(|ui| task.ui(ui));
            }
        });
    }

    /// Collection name.
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

    pub(crate) fn push_task(&mut self, task: AnyTask) {
        self.executor.push(task);
    }

    /// Сalls all handle-methods, in this order:
    /// - [`handle_execution`](Self::handle_execution)
    /// - [`handle_progress`](Self::handle_progress)
    /// - [`handle_results`](Self::handle_results)
    /// - [`handle_deletion`](Self::handle_deletion)
    pub fn handle_all(&mut self, result_handle: AnyHandler<'_>) {
        self.handle_execution();
        self.handle_progress();
        self.handle_results(result_handle);
        self.handle_deletion();
    }

    /// Handles tasks execution results using provided handle.
    pub fn handle_results(&mut self, mut handle: AnyHandler<'_>) {
        if let Ok(value) = self.channel.receiver().try_recv() {
            handle.apply(value)
        }
    }

    /// Handles tasks deletion.
    pub fn handle_deletion(&mut self) {
        self.tasks.retain(|task| !task.is_finished())
    }

    /// Handles tasks progress.
    pub fn handle_progress(&mut self) {
        for progress in self.tasks.iter_mut().filter_map(|task| task.progress_mut()) {
            if let Ok(data) = progress.receiver().try_recv() {
                data.apply(progress.current_mut())
            }
        }
    }

    /// Handles tasks execution.
    ///
    /// More specifically it calls [`TasksExecutor::poll`](crate::TaskExecutor) method
    /// which decides if the task is ready to be executed. This method will call `poll`
    /// as long as it returns [`ExecutionPoll::Ready`](crate::ExecutionPoll) which means
    /// that there's still tasks to execute. If [`ExecutionPoll::Pending`](crate::ExecutionPoll)
    /// is returned it will stop the polling.
    pub fn handle_execution(&mut self) {
        use crate::execution::ExecutionPoll as E;
        while let E::Ready(task) = self.executor.poll(&self.tasks) {
            self.execute(task)
        }
    }
}
