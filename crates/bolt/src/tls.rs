use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use rustls::ServerConfig;
use std::sync::Arc;

pub struct CertResolver {}

impl ResolvesServerCert for CertResolver {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        None
    }
}

pub fn mk_config() -> Arc<ServerConfig> {
    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(CertResolver {}));

    config.alpn_protocols = vec![b"http/1.1".to_vec(), b"h2".to_vec()];

    Arc::new(config)
}
