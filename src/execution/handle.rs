use std::any::Any;

use crate::any::{IntoAny, HigherKinded};

pub type AnyHandle<'h> = Handle<'h, Box<dyn Any + Send>>;

pub struct Handle<'h, T>(Box<dyn FnMut(T) + 'h>);

impl<'h, T: 'static> Handle<'h, T> {
    pub fn new(handle: impl FnMut(T) + 'h) -> Self {
        Self(Box::new(handle))
    }

    pub fn apply(&mut self, value: T) {
        (self.0)(value)
    }
}

impl<'h, T> HigherKinded for Handle<'h, T> {
    type T<A> = Handle<'h, A>;
}

impl<'h, T> IntoAny for Handle<'h, T>
where
    T: 'static,
{
    fn into_any(mut self) -> Self::T<Box<dyn Any + Send>> {
        let handle = Box::new(move |boxed_any: Box<dyn Any + Send>| {
            let reference = boxed_any.downcast::<T>().unwrap();
            (self.0)(*reference)
        });

        Handle(handle)
    }
}
