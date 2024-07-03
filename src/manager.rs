use std::{
    any::{type_name, TypeId},
    collections::HashMap,
};

use crate::{any::IntoAny, TaskExecutor};

use super::{
    collection::{CollectionData, TasksCollection},
    task::Task,
};

/// It holds all collections and a handle that is used to handle the progress of the execution.
///
/// The generic `P` determines the type that is used to track the execution progress of all tasks.
///
///
/// ```rust
/// # use egui_task_manager::TaskManager;
/// struct MyState {
///     // Since we passed () as a parameter now all your tasks that has progress tracking
///     // must use ().
///     manager: TaskManager<()>,
///     // ...
/// }
/// ```
pub struct TaskManager<P> {
    collections: HashMap<TypeId, CollectionData<P>>,
    progress_handle: fn(&mut u32, P),
}

impl<P> TaskManager<P>
where
    P: 'static,
{
    #[cfg(feature = "egui")]
    /// Draws a simple ui.
    pub fn ui(&self, ui: &mut egui::Ui) {
        for collection in self.collections.values() {
            collection.ui(ui)
        }
    }

    /// Creates a new instance of the manager using provided progress handle.
    pub fn new(handle: fn(&mut u32, P)) -> Self {
        Self {
            collections: HashMap::new(),
            progress_handle: handle,
        }
    }

    fn get_collection_mut<'c, C>(&mut self) -> &mut CollectionData<P>
    where
        C: TasksCollection<'c, P> + 'static,
    {
        self.collections
            .get_mut(&TypeId::of::<C>())
            .unwrap_or_else(move || {
                panic!(
                    "You must add `{}` collection to the `TaskManager` by calling `add_collection`",
                    type_name::<C>()
                )
            })
    }

    /// Adds a new collection and handles its results.
    ///
    /// It must be called in the beginning of the update function.
    pub fn add_collection<'c, C>(&mut self, context: C::Context) -> &mut Self
    where
        C: TasksCollection<'c, P> + 'static,
        C::Executor: TaskExecutor<P> + 'static,
    {
        self.push_collection::<C>().handle_collection::<C>(context)
    }

    /// Adds a new collection.
    ///
    /// It is recommended to use [`add_collection`](Self::add_collection).
    pub fn push_collection<'c, C>(&mut self) -> &mut Self
    where
        C: TasksCollection<'c, P> + 'static,
        C::Executor: TaskExecutor<P> + 'static,
    {
        let id = TypeId::of::<C>();

        if self.collections.contains_key(&id) {
            return self;
        }

        self.collections
            .insert(id, CollectionData::from_collection::<C>());
        self
    }

    /// Handles the tasks of the specified collection.
    ///
    /// It is recommended to use [`add_collection`](Self::add_collection).
    pub fn handle_collection<'c, C>(&mut self, context: C::Context) -> &mut Self
    where
        C: TasksCollection<'c, P> + 'static,
    {
        let handle = C::handle(context).into_any();
        let progress_handle = self.progress_handle;
        self.get_collection_mut::<C>()
            .handle_all(handle, progress_handle);

        self
    }

    /// Pushes a task to the executor of the specified collection.
    pub fn push_task<'c, C>(&mut self, task: Task<C::Target, P>)
    where
        C: TasksCollection<'c, P> + 'static,
        C::Target: Send + 'static,
    {
        self.get_collection_mut::<C>().push_task::<C>(task);
    }
}
