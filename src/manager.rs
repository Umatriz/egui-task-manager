use std::{
    any::{type_name, TypeId},
    collections::HashMap,
};

use crate::{any::IntoAny, TaskExecutor};

use super::{
    collection::{CollectionData, TasksCollection},
    task::Task,
};

pub struct TaskManager<P> {
    collections: HashMap<TypeId, CollectionData<P>>,
    progress_handle: fn(&mut u32, P),
}

impl<P> TaskManager<P>
where
    P: 'static,
{
    #[cfg(feature = "egui")]
    pub fn ui(&self, ui: &mut egui::Ui) {
        for collection in self.collections.values() {
            collection.ui(ui)
        }
    }

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

    pub fn collection<'c, C>(&mut self, context: C::Context) -> &mut Self
    where
        C: TasksCollection<'c, P> + 'static,
        C::Executor: TaskExecutor<P> + 'static,
    {
        self.add_collection::<C>().handle_collection::<C>(context)
    }

    pub fn add_collection<'c, C>(&mut self) -> &mut Self
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

    pub fn push_task<'c, C>(&mut self, task: Task<C::Target, P>)
    where
        C: TasksCollection<'c, P> + 'static,
        C::Target: Send + 'static,
    {
        self.get_collection_mut::<C>().push_task::<C>(task);
    }
}
