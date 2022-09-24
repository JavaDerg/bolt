use bolt_router::DomainRouter;
use std::collections::HashMap;

pub trait ConfigProvider: 'static + Send + Sync {
    fn life_reloads(&self) -> bool;

    fn hosts(&self) -> Box<dyn VHostsProvider>;
}

pub trait VHostsProvider: 'static + Send + Sync {
    fn vhosts(&self) -> DomainRouter;
    fn default(&self) -> Option<Box<dyn VHost>>;
}

pub trait VHost: 'static + Send + Sync {}
