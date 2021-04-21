use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::thread::ThreadId;

pub struct LocalStore<T, F>
where
    T: Send,
    F: Fn() -> Box<dyn Future<Output = T>>,
{
    default: F,
    storage: HashMap<ThreadId, Arc<T>>,
}

impl<T, F> LocalStore<T, F>
where
    T: Send,
    F: Fn() -> Box<dyn Future<Output = T>>,
{
}
