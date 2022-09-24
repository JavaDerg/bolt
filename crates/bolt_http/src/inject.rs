use std::any::Any;
use crate::resources::Resources;

pub trait FromResources: Clone + Any + Send + Sync + 'static {
    fn from_res(res: &Resources) -> Option<Self> {
        res.query::<Self>()
    }
}

default impl<T: Clone + Any + Send + Sync + 'static> FromResources for T {
}

impl FromResources for () {
    fn from_res(res: &Resources) -> Option<Self> {
        Some(())
    }
}

