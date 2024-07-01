use std::future::Future;

/// The handle that allows the task to be aborted
pub struct TaskHandle(tokio::task::JoinHandle<()>);

impl<Fut> From<Fut> for TaskHandle
where
    Fut: Future<Output = ()> + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn from(value: Fut) -> Self {
        Self(tokio::spawn(value))
    }
}

impl TaskHandle {
    pub fn abort(&self) {
        self.0.abort()
    }
}
