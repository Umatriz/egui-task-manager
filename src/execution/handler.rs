use std::any::Any;

use crate::any::{HigherKinded, IntoAny};

/// Handler that has `Box<dyn Any + Send>` as a type parameter.
pub type AnyHandler<'h> = Handler<'h, Box<dyn Any + Send>>;

/// Handler that is used to handle task's result.
pub struct Handler<'h, T>(Box<dyn FnMut(T) + 'h>);

impl<'h, T: 'static> Handler<'h, T> {
    /// Creates a new handle.
    pub fn new(handler: impl FnMut(T) + 'h) -> Self {
        Self(Box::new(handler))
    }

    /// Applies handle on some value
    pub fn apply(&mut self, value: T) {
        (self.0)(value)
    }
}

impl<'h, T> HigherKinded for Handler<'h, T> {
    type T<A> = Handler<'h, A>;
}

impl<'h, T> IntoAny for Handler<'h, T>
where
    T: 'static,
{
    fn into_any(mut self) -> Self::T<Box<dyn Any + Send>> {
        let handler = Box::new(move |boxed_any: Box<dyn Any + Send>| {
            let reference = boxed_any.downcast::<T>().unwrap();
            (self.0)(*reference)
        });

        Handler(handler)
    }
}
