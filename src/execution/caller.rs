use std::{any::Any, future::Future};

use crate::any::{HigherKinded, IntoAny};

use super::{progress::TaskProgressShared, PinnedFuture};

/// The task's body itself.
///
/// It has two states.
/// - `Standard` has no progress.
/// - `Progressing` has a progress and provides [`TaskProgressShared`][crate::TaskProgressShared].
pub enum Caller<T> {
    /// Standard caller. No progress just a future.
    Standard(PinnedFuture<T>),

    /// Progressing caller. Has progress. Holds a closure that returns a future.
    Progressing(Box<dyn FnOnce(TaskProgressShared) -> PinnedFuture<T>>),
}

impl<T> Caller<T> {
    /// Create a [`Standard`](Self::Standard) caller from a future.
    pub fn standard<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = T> + Send + 'static,
    {
        Self::Standard(Box::pin(fut))
    }

    /// Create a [`Standard`](Self::Progressing) caller from a closure that returns a future.
    pub fn progressing<F, Fut>(fun: F) -> Self
    where
        F: FnOnce(TaskProgressShared) -> Fut + 'static,
        Fut: Future<Output = T> + Send + 'static,
    {
        Self::Progressing(Box::new(|progress| Box::pin((fun)(progress))))
    }
}

impl<T> HigherKinded for Caller<T> {
    type T<A> = Caller<A>;
}

impl<U> IntoAny for Caller<U>
where
    U: Send + 'static,
{
    fn into_any(self) -> Self::T<Box<dyn Any + Send>> {
        match self {
            Caller::Standard(fut) => {
                Caller::standard(async { Box::new(fut.await) as Box<dyn Any + Send> })
            }
            Caller::Progressing(fun) => {
                let fun = Box::new(|progress| {
                    let fut = (fun)(progress);
                    Box::pin(async move { Box::new(fut.await) as Box<dyn Any + Send> })
                });

                Caller::progressing(fun)
            }
        }
    }
}
