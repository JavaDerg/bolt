use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::sync::Arc;
use typemap::{ShareCloneMap, SendMap, Key};

pub struct Resources {
    prev: Option<Arc<Self>>,

    take: SendMap,
    local: ShareCloneMap,
}

struct LocalKey<T>(PhantomData<T>);

impl<T: Any> Key for LocalKey<T> {
    type Value = T;
}

impl<T: Clone> Clone for LocalKey<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Resources {
    pub fn new() -> Self {
        Self {
            prev: None,
            take: SendMap::custom(),
            local: ShareCloneMap::custom(),
        }
    }

    pub fn new_with(prev: Arc<Self>) -> Self {
        Self {
            prev: Some(prev),
            take: SendMap::custom(),
            local: ShareCloneMap::custom(),
        }
    }

    pub fn query<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        if let val @ Some(_) = self.local.get::<LocalKey<T>>().cloned() {
            val
        } else if let Some(prev) = &self.prev {
            prev.query::<T>()
        } else {
            None
        }
    }

    pub fn query_take<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.take.remove::<LocalKey<T>>()
    }
}
