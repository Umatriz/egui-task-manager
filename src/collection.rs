use std::any::Any;

use crate::{
    any::IntoAny,
    channel::Channel,
    execution::{AnyHandle, Handle, TaskExecutor},
    task::{AnyTask, Task, TaskData},
};

pub trait TasksCollection<'c, P> {
    type Context: 'c;
    type Target: Send + 'static;
    type Executor: TaskExecutor<P> + Default;

    fn name() -> &'static str;
    fn handle(context: Self::Context) -> Handle<'c, Self::Target>;
}

pub struct CollectionData<P> {
    name: &'static str,
    channel: Channel<Box<dyn Any + Send>>,
    tasks: Vec<TaskData<P>>,
    executor: Box<dyn TaskExecutor<P>>,
}

impl<P> CollectionData<P>
where
    P: 'static,
{
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
        C: TasksCollection<'c, P>,
        C::Executor: 'static,
    {
        Self {
            name: C::name(),
            channel: Channel::new(),
            tasks: Vec::new(),
            executor: Box::<C::Executor>::default(),
        }
    }

    fn execute(&mut self, task: AnyTask<P>) {
        let sender = self.channel.sender();
        let task_data = task.execute(sender);
        self.push_task_data(task_data);
    }

    fn push_task_data(&mut self, task_data: TaskData<P>) {
        self.tasks.push(task_data)
    }

    pub fn push_task<'c, C>(&mut self, task: Task<C::Target, P>)
    where
        C: TasksCollection<'c, P>,
        C::Target: Send,
    {
        self.executor.push(task.into_any());
    }

    pub fn handle_all(&mut self, result_handle: AnyHandle<'_>, progress_handle: fn(&mut u32, P)) {
        self.handle_execution();
        self.handle_progress(progress_handle);
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

    pub fn handle_progress(&mut self, progress_handle: fn(&mut u32, P)) {
        for progress in self.tasks.iter_mut().filter_map(|task| task.progress_mut()) {
            if let Ok(result) = progress.receiver().try_recv() {
                progress_handle(progress.current_mut(), result)
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
