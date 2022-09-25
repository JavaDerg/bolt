use std::net::IpAddr;

pub struct Domain {
    pub listeners: Vec<Listener>,
}

pub struct Listener {
    pub address: IpAddr,
    pub port: u16,

    pub tls: bool,

    pub http1: bool,
    pub http2: bool,
}
