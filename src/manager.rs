use std::{
    any::{type_name, TypeId},
    collections::HashMap,
};

use crate::{any::IntoAny, TaskExecutor};

use super::{
    collection::{CollectionData, TasksCollection},
    task::Task,
};

/// It holds all collections.
///
/// ```rust
/// # use egui_task_manager::TaskManager;
/// struct MyState {
///     // Keep it in your app's state.
///     manager: TaskManager,
///     // ...
/// }
/// ```
#[derive(Default)]
pub struct TaskManager {
    collections: HashMap<TypeId, CollectionData>,
}

impl TaskManager {
    #[cfg(feature = "egui")]
    /// Draws a simple ui.
    pub fn ui(&self, ui: &mut egui::Ui) {
        for collection in self.collections.values() {
            collection.ui(ui)
        }
    }

    /// Creates a new instance of the manager using provided progress handle.
    pub fn new() -> Self {
        Self::default()
    }

    fn get_collection_mut<'c, C>(&mut self) -> &mut CollectionData
    where
        C: TasksCollection<'c> + 'static,
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
        C: TasksCollection<'c> + 'static,
        C::Executor: TaskExecutor + 'static,
    {
        self.push_collection::<C>().handle_collection::<C>(context)
    }

    /// Adds a new collection. It **does not** handle the results, progression, execution and deletion.
    /// If you use this method you **must** call [`handle_collection`](Self::handle_collection) to
    /// see the anything.
    ///
    /// It is recommended to use [`add_collection`](Self::add_collection).
    fn push_collection<'c, C>(&mut self) -> &mut Self
    where
        C: TasksCollection<'c> + 'static,
        C::Executor: TaskExecutor + 'static,
    {
        let id = TypeId::of::<C>();

        if self.collections.contains_key(&id) {
            return self;
        }

        self.collections
            .insert(id, CollectionData::from_collection::<C>());
        self
    }

    /// Handles the tasks of the specified collection. It **does not** add a new collection
    /// to the manager. If you want to use this method you **must** call [`push_collection`](Self::push_collection).
    ///
    /// It is recommended to use [`add_collection`](Self::add_collection).
    fn handle_collection<'c, C>(&mut self, context: C::Context) -> &mut Self
    where
        C: TasksCollection<'c> + 'static,
    {
        let handle = C::handle(context).into_any();

        self.get_collection_mut::<C>().handle_all(handle);
        self
    }

    /// Pushes a task to the executor of the specified collection.
    pub fn push_task<'c, C>(&mut self, task: Task<C::Target>)
    where
        C: TasksCollection<'c> + 'static,
        C::Target: Send + 'static,
    {
        self.get_collection_mut::<C>().push_task(task.into_any());
    }
}
