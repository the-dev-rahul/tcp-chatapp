use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
struct Client {
    sender: UnboundedSender<Message>,
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.sender.same_channel(&other.sender)
    }
}

impl Eq for Client {}

impl Hash for Client {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sender.same_channel(&self.sender).hash(state);
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.expect("Failed to bind to address");
    let clients = Arc::new(Mutex::new(HashSet::new()));

    while let Ok((stream, _)) = listener.accept().await {
        let clients = Arc::clone(&clients);

        tokio::spawn(handle_connection(stream, clients));
    }
}

async fn handle_connection(
    stream: TcpStream,
    clients: Arc<Mutex<HashSet<Client>>>
) {
    let ws_stream = accept_async(stream).await.expect("Failed to accept WebSocket connection");
    let (mut write, mut read) = ws_stream.split();
    let (client_tx, mut client_rx) = tokio::sync::mpsc::unbounded_channel();

    {
        let mut clients = clients.lock().await;
        clients.insert(Client { sender: client_tx });
    }

    let write_task = tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    let read_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {

            let clients = clients.lock().await;
            for client in clients.iter() {
                let _ = client.sender.send(msg.clone());
            }
        }
    });

    let _ = tokio::try_join!(write_task, read_task);
}
