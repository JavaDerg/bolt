use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, Read};
use std::path::Path;
use std::sync::Arc;

use async_rustls::rustls::ClientHello;
use async_rustls::rustls::sign::CertifiedKey;
use rustls::ResolvesServerCert;

use crate::map_maybe::MapMaybeExt;
use crate::router::Router;

pub struct ServerConfig {
    default: Arc<DomainSpecificConfig>,
    domain_specific: HashMap<DomainKey<'static>, Arc<DomainSpecificConfig>>,
}

pub struct DomainSpecificConfig {
    cert: CertifiedKey,
    router: Arc<Router>,
}

pub struct ServerConfigBuilder(ServerConfig);

#[derive(Eq)]
pub enum DomainKey<'a> {
    Owned(Vec<u8>),
    Shared(&'a [u8]),
}

struct FakeReader<'a>(&'a [u8], usize);

impl ServerConfig {
    pub fn builder(default_dsc: Arc<DomainSpecificConfig>) -> ServerConfigBuilder {
        ServerConfigBuilder(ServerConfig {
            default: default_dsc,
            domain_specific: Default::default(),
        })
    }

    pub fn default(&self) -> Arc<DomainSpecificConfig> {
        self.default.clone()
    }

    pub fn get(&self, domain: &[u8]) -> Arc<DomainSpecificConfig> {
        self.domain_specific
            .get(&DomainKey::Shared(domain))
            .or(Some(&self.default))
            .unwrap()
            .clone()
    }
}

impl ResolvesServerCert for ServerConfig {
    fn resolve(&self, client_hello: ClientHello) -> Option<CertifiedKey> {
        client_hello
            .server_name()
            .map_maybe(|name| {
                let key = DomainKey::Shared(unsafe { std::mem::transmute(name) }); // FIXME: This sucks
                let cfg = self.domain_specific.get(&key)?;
                Some(cfg.cert.clone())
            })
            .or_else(|| Some(self.default.cert.clone()))
    }
}

impl ServerConfigBuilder {
    pub fn register(
        &mut self,
        domain: impl Into<String>,
        dsc: &Arc<DomainSpecificConfig>,
    ) -> &mut Self {
        self.0
            .domain_specific
            .insert(DomainKey::Owned(domain.into().into_bytes()), dsc.clone());
        self
    }

    pub fn finish(self) -> Arc<ServerConfig> {
        Arc::new(self.0)
    }
}

impl DomainSpecificConfig {
    pub fn new(cert: CertifiedKey, router: Router) -> Arc<Self> {
        Arc::new(Self {
            cert,
            router: Arc::new(router),
        })
    }

    pub fn router(&self) -> Arc<Router> {
        self.router.clone()
    }
}

impl<'a> DomainKey<'a> {
    fn as_ref(&'a self) -> &'a [u8] {
        match self {
            DomainKey::Owned(vec) => vec.as_slice(),
            DomainKey::Shared(slice) => *slice,
        }
    }
}

impl<'a, 'b> PartialEq<DomainKey<'b>> for DomainKey<'a> {
    fn eq(&self, other: &DomainKey<'b>) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<'a> Hash for DomainKey<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl<'a> Read for FakeReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.1 >= self.0.len() {
            return Ok(0);
        }
        let available = self.fill_buf().unwrap();
        let copy = available.len().min(buf.len());
        (&mut buf[..copy]).copy_from_slice(&available[..copy]);
        Ok(copy)
    }
}

impl<'a> BufRead for FakeReader<'a> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        Ok(&self.0[self.1.min(self.0.len())..])
    }

    fn consume(&mut self, amt: usize) {
        self.1 += amt;
    }
}

pub fn load_cert_key(certs: &Path, key: &Path) -> CertifiedKey {
    let certs = rustls_pemfile::certs(&mut FakeReader(std::fs::read(certs).unwrap().as_slice(), 0))
        .unwrap()
        .into_iter()
        .map(rustls::Certificate)
        .collect::<Vec<_>>();

    let private = {
        loop {
            match rustls_pemfile::read_one(&mut FakeReader(
                std::fs::read(key).unwrap().as_slice(),
                0,
            ))
                .unwrap()
            {
                Some(rustls_pemfile::Item::RSAKey(key)) => break rustls::PrivateKey(key),
                Some(rustls_pemfile::Item::PKCS8Key(key)) => break rustls::PrivateKey(key),
                None => panic!("Could not find valid key"),
                _ => {}
            }
        }
    };
    CertifiedKey {
        cert: certs,
        key: Arc::new(rustls::sign::any_supported_type(&private).unwrap()),
        ocsp: None,
        sct_list: None,
    }
}
