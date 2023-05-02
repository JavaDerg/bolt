use crate::location::router::PathRouter;
use crate::matchers::{PrefixMatcher, RegexMatcher};
use hashbrown::HashMap;

pub struct RouterBuilder<S> {
    exact: HashMap<String, S>,
    prefix: Vec<(String, S)>,
    regex: Vec<(String, S)>,
    default: S,
}

#[derive(Debug, thiserror::Error)]
pub enum RouterBuildError {
    #[error("regex set build error: {0}")]
    Regex(#[from] regex::Error),
    #[error("aho corasick build error: {0}")]
    AhoCorasick(#[from] aho_corasick::BuildError),
}

impl<S> RouterBuilder<S> {
    pub fn with_default(s: S) -> Self {
        Self {
            exact: HashMap::new(),
            prefix: Vec::new(),
            regex: Vec::new(),
            default: s,
        }
    }

    pub fn add_exact(mut self, path: impl Into<String>, s: S) -> Self {
        self.exact.insert(path.into(), s);
        self
    }

    pub fn add_prefix(mut self, path: impl Into<String>, s: S) -> Self {
        self.prefix.push((path.into(), s));
        self
    }

    pub fn add_regex(mut self, path: impl Into<String>, s: S) -> Self {
        self.regex.push((path.into(), s));
        self
    }

    pub fn build(
        Self {
            exact,
            prefix,
            regex,
            default,
        }: Self,
    ) -> Result<(PathRouter, Vec<S>), RouterBuildError> {
        let mut routes = vec![default];

        let exact = exact
            .into_iter()
            .enumerate()
            .map(|(i, (path, s))| {
                routes.push(s);
                (path, i + 1)
            })
            .collect::<HashMap<_, _>>();

        let last_index = routes.len();

        let (prefixes, services): (Vec<_>, Vec<_>) = prefix.into_iter().unzip();
        routes.extend(services);

        let prefix = PrefixMatcher::new(
            prefixes,
            (last_index..last_index + prefixes.len()).collect(),
        )?;

        let last_index = routes.len();

        let (regexes, services): (Vec<_>, Vec<_>) = regex.into_iter().unzip();
        routes.extend(services);

        let regex = RegexMatcher::new(regexes, (last_index..last_index + regexes.len()).collect())?;

        Ok((
            PathRouter {
                exact,
                prefix,
                regex,
            },
            routes,
        ))
    }
}
