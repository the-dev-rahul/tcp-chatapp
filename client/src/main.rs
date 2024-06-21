use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt, SinkExt};
use url::Url;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let url = Url::parse("ws://127.0.0.1:8080").unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    let client_id = Uuid::new_v4().to_string();

    let client_id_clone = client_id.clone();
    let write_task = tokio::spawn(async move {
        use tokio::io::{self, AsyncBufReadExt};

        let stdin = io::BufReader::new(io::stdin());
        let mut lines = stdin.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let msg = format!("{}: {}", client_id_clone, line);
            if write.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let client_id_clone = client_id.clone();
    let read_task = tokio::spawn(async move {
      while let Some(Ok(message)) = read.next().await {
            if let Message::Text(txt) = message {
                let parts: Vec<&str> = txt.splitn(2, ": ").collect();
                if parts.len() == 2 && parts[0] != client_id_clone {
                    println!("Received: {}", parts[1]);
                }
            }
        }
    });

    let _ = tokio::try_join!(write_task, read_task);
}
