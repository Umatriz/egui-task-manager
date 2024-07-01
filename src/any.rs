use std::any::Any;

pub(crate) trait HigherKinded {
    type T<A>;
}

pub(crate) trait IntoAny: HigherKinded {
    fn into_any(self) -> Self::T<Box<dyn Any + Send>>;
}
