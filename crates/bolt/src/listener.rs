use flume::Sender;
use tokio::task::JoinHandle;

pub struct ListenerHandle {
    jh: JoinHandle<eyre::Result<()>>,
    tx: Sender<ListenerMessage>,
}

enum ListenerMessage {
    Close,
}

/*

listener task will handle all tcp listeners
-> peek first bytes
-> check for tls
-> check config if plain/tls is allowed for the listener

for tls attach SNI to request extensions

 */
