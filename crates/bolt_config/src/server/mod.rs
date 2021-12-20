//! # Server config module
//!
//! ## Domain resolution:
//! ```not_rust
//! exact match
//!   -> aho corasick matching
//!     -> regex matching
//!       -> default route
//! ```
//!

mod error;

use aho_corasick::AhoCorasick;
use dashmap::DashMap;
pub use error::ParseError;
use regex::RegexSet;
use std::net::IpAddr;
use std::sync::Arc;

/// Fast concurrent hashmap
type Fchm<K, V> = DashMap<K, V, fxhash::FxBuildHasher>;

#[derive(Clone)]
pub struct ServerConfig(Arc<InnerServerConfig>);
struct InnerServerConfig {
    services: Fchm<HostPortProtocolKey, DomainResolvedConfigs>,
}

#[derive(Clone)]
pub struct DomainResolvedConfigs(Arc<InnerDomainResolvedConfigs>);
struct InnerDomainResolvedConfigs {
    exact: Fchm<String, SiteSpecificConfig>,

    aho_coresick: Option<AhoCorasick>,
    aho_coresick_resolution: Vec<SiteSpecificConfig>,

    regex_set: Option<RegexSet>,
    regex_resolution: Vec<SiteSpecificConfig>,

    default: Option<SiteSpecificConfig>,
}

impl DomainResolvedConfigs {
    pub fn resolve(&self, domain: &str) -> Option<SiteSpecificConfig> {
        self.0
            .exact
            .get(domain)
            .map(|rf| rf.value().clone())
            .or_else(|| self.aho_coresick_resolve(domain))
            .or_else(|| self.regex_resolve(domain))
            .or_else(|| self.0.default.clone())
    }

    fn aho_coresick_resolve(&self, domain: &str) -> Option<SiteSpecificConfig> {
        /*
        quick disclaimer, aho corasick is a multi string search algorithm
        that means we specify a certain amount of matchable strings, we call that set S and a string we want to search X
        the aho corasick algorithm will try to find any match of the members of S in X
        aho corasick has a time complexity of O(len(X) + len(S) + len(matches))
        */

        let ac = self.0.aho_coresick.as_ref()?;
        let mut res_mtch = None;

        for mtch in ac.find_iter(domain) {
            // This check makes sure the matching domain is aligned to the end of the input
            if mtch.end() != domain.len() {
                continue;
            }
            // if the start of the match is 0 it means we have a exact match
            else if mtch.start() == 0 {
                res_mtch = Some(mtch);
                break;
            }
            // if it isn't a exact match we check if we matched to a higher level domain
            // by verifying if the domain starts with a dot before the match
            else if domain.as_bytes()[mtch.start() - 1] != b'.' {
                continue;
            }
            // if all is true we will select the lowest level match, that means
            // `sub.domain.tld` will be preferred over `domain.tld`
            // this check only applies if we already found a match
            else if let Some(res) = &res_mtch {
                if mtch.start() < res.start() {
                    res_mtch = Some(mtch);
                }
            } else {
                res_mtch = Some(mtch);
                continue;
            }
        }
        let mtch = res_mtch?;

        self.0.aho_coresick_resolution.get(mtch.pattern()).cloned()
    }

    fn regex_resolve(&self, domain: &str) -> Option<SiteSpecificConfig> {
        let rx = self.0.regex_set.as_ref()?;
        let index = rx.matches(domain).iter().next()?;
        self.0.regex_resolution.get(index).cloned()
    }
}

#[derive(Clone)]
pub struct SiteSpecificConfig {}
struct InnerSiteSpecificConfig {}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum HostPortProtocolKey {
    AnyHost {
        port: u16,
        protocol: InitProtocol,
    },
    Specific {
        ip: IpAddr,
        port: u16,
        protocol: InitProtocol,
    },
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
#[non_exhaustive]
pub enum InitProtocol {
    Http,
    Tls,
}

impl ServerConfig {
    pub fn resolve_unspecific(
        &self,
        port: u16,
        protocol: InitProtocol,
    ) -> Option<DomainResolvedConfigs> {
        self.0
            .services
            .get(&HostPortProtocolKey::AnyHost { port, protocol })
            .map(|rf| rf.value().clone())
    }

    pub fn resolve_specific(
        &self,
        ip: IpAddr,
        port: u16,
        protocol: InitProtocol,
    ) -> Option<DomainResolvedConfigs> {
        self.0
            .services
            .get(&HostPortProtocolKey::Specific { ip, port, protocol })
            .map(|rf| rf.value().clone())
    }

    pub fn resolve_specific_with_fallback(
        &self,
        ip: IpAddr,
        port: u16,
        protocol: InitProtocol,
    ) -> Option<DomainResolvedConfigs> {
        self.resolve_specific(ip, port, protocol)
            .or_else(|| self.resolve_unspecific(port, protocol))
    }
}

/*
impl ServerConfig {
    /// TODO
    /// This panics
    pub async fn load_from(fs: impl VirtFs) -> Result<Self, ParseError> {
        let mut file = fs.open(Path::new("bolt.conf")).await?;
        let mut cfg_string = String::with_capacity(1024);
        let _ = file.read_to_string(&mut cfg_string).await?;

        let lexer = super::parser::lexer::lex(&cfg_string).collect::<Vec<_>>();

        let mut tokens = Vec::with_capacity(lexer.len());
        let mut fails = vec![];

        for token in lexer {
            match token {
                Ok(token) => tokens.push(token),
                Err(err) => fails.push(err),
            }
        }

        if !fails.is_empty() {
            return Err(ParseError::Parse(ErrorBundle(fails)));
        }

        let mut commands = super::parser::syntax::analyze(tokens.into_iter());

        for cmd in commands {
            println!("{:?}", &cmd);
        }

        todo!()
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {}
    }
}

*/
