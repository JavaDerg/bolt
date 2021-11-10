use crate::data::{Request, ResponseBuilder};
use crate::middleware::{Middleware, MiddlewareAction};
use crate::responses;
use regex::RegexSet;
use std::borrow::Cow;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct Router {
    dsc: HashMap<(Cow<'static, str>, SocketAddr), Arc<Routes>>,
    default: Option<Routes>,
}

pub struct Routes {
    default: Route,
    tree: Option<HashMap<Cow<'static, str>, Routes>>,
    regex: Option<RegexSet>,
    regex_routes: Vec<Routes>,
}

#[derive(Clone)]
pub enum Route {
    None,
    Middleware(Arc<dyn Middleware + Send + Sync>),
}

impl Middleware for Router {
    fn process(self: Arc<Self>, req: Request, rb: &mut ResponseBuilder) -> MiddlewareAction {
        let path = (*req.path).segments.as_ref();

        let routes = match self.dsc.get(req.domain.as_str()).or(self.default.as_ref()) {
            Some(routes) => routes,
            None => return MiddlewareAction::Direct(None),
        };
        match routes.r#match(path) {
            Route::None => MiddlewareAction::Direct(Some(responses::_404(&req))),
            Route::Middleware(layer) => layer.process(req, rb),
        }
    }
}

impl Routes {
    pub fn r#match(&self, path: &[Cow<'_, str>]) -> Route {
        if path.len() == 0 {
            return self.default.clone();
        }
        let segment = &path[0];
        if let Some(tree) = &self.tree {
            if let Some(route) = tree.get(segment) {
                return route.r#match(&path[1..]);
            }
        }
        if let Some(regex) = &self.regex {
            if let Some(index) = regex.matches(segment.as_ref()).iter().next() {
                return self
                    .regex_routes
                    .get(index)
                    .expect("Invalid regex set index")
                    .r#match(&path[1..]);
            }
        }
        self.default.clone()
    }
}
