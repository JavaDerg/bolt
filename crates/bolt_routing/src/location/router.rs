use crate::location::RouterBuilder;
use crate::matchers::{PrefixMatcher, RegexMatcher};
use arc_swap::ArcSwap;
use hashbrown::HashMap;
use http::Request;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tower::steer::Picker;

pub struct ArcPathRouter(Arc<PathRouter>);

pub struct PathRouter {
    pub(super) exact: HashMap<String, usize>,
    pub(super) prefix: PrefixMatcher<'/'>,
    pub(super) regex: RegexMatcher,
}

pub struct LastRouteMatch {
    end: AtomicUsize,
}

impl PathRouter {
    pub fn builder<S>(default: S) -> RouterBuilder<S> {
        RouterBuilder::with_default(default)
    }

    pub fn pick_immu<B>(&self, r: &Request<B>) -> usize {
        let path = r.uri().path();

        let (idx, end) = self
            .exact
            .get(path)
            .map(|i| (*i, path.len()))
            .or_else(|| self.prefix.match_(path).map(|m| (m.index, m.end)))
            .or_else(|| self.regex.match_(path).map(|i| (i, path.len())))
            .unwrap_or((0, 0));

        if let Some(ext) = r.extensions().get::<LastRouteMatch>() {
            ext.end.store(end, Ordering::Relaxed);
        }

        idx
    }
}

impl<S, B> Picker<S, Request<B>> for PathRouter {
    fn pick(&mut self, r: &Request<B>, _services: &[S]) -> usize {
        self.pick_immu(r)
    }
}
