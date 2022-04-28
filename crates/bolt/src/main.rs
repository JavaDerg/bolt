use hyper::body::HttpBody;
use std::net::{SocketAddr:w
};
use tokio::net::{lookup_host, TcpListener, TcpStream};

mod cli;

fn main() {
    let cpus = num_cpus::get();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(cpus)
        .build()
        .expect("Unable to build tokio runtime")
        .block_on(start());
}

async fn start() {}

#[derive(thiserror::Error, Debug)]
enum ListenError {
    #[error("Unable to bind to {0}")]
    BindFailure(std::io::Error, SocketAddr),
    #[error("Unable to listen on {0}")]
    AcceptFailure(std::io::Error, SocketAddr),
}

async fn listen(addr: SocketAddr) -> Result<(), ListenError> {
    let mut listener = TcpListener::bind(addr)
        .await
        .map_err(|err| ListenError::BindFailure(err, addr))?;

    loop {
        let (mut stream, addr) = listener
            .accept()
            .await
            .map_err(|err| ListenError::AcceptFailure(err, addr))?;

        tokio::spawn(async move {
            let tls = check_for_tls(&mut stream).await;
        });
    }

    Ok(())
}

async fn check_for_tls(stream: &mut TcpStream) -> std::io::Result<bool> {
    let mut buf = [0; 1];
    stream.peek(&mut buf[..]).await?;

    Ok(buf[0] == 0x16)
}
