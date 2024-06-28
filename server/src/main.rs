use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use std::collections::HashMap; 
use std::sync::Arc;
use uuid::Uuid;

type Clients = Arc<Mutex<HashMap<String,mpsc::UnboundedSender<String>>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let clients = Arc::clone(&clients);
        
        tokio::spawn(handle_client(socket, clients.clone()));
    }
}

async fn handle_client(socket: TcpStream, clients: Clients) {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let (reader, mut writer) = socket.into_split();
    let client_id = Uuid::new_v4().to_string();
    
    
    {   
        let client_id = client_id.clone();
        let mut clients = clients.lock().await;
        clients.insert(client_id , tx.clone());
    }
    
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if writer.write_all(msg.as_bytes()).await.is_err() {
                break;
            }
        }
    });


    let clients_clone = clients.clone();
    let client_id_clone = client_id.clone();
    let read_task = tokio::spawn(async move {
        let mut reader = BufReader::new(reader);
        let mut buf = String::new();
        loop {
            buf.clear();
            match reader.read_line(&mut buf).await {
                Ok(0) => break, 
                Ok(_) => {
                    print!("Received: {buf}");
                    let clients = clients_clone.lock().await;
                    for (cur_client_id, client) in clients.iter() {
                        if *cur_client_id != client_id_clone{
                            let _ = client.send(buf.clone());
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    let server_write_task = tokio::spawn(async move{
        let mut stdin = BufReader::new(io::stdin());
        let mut line = String::new();
        loop {
            line.clear();
            stdin.read_line(&mut line).await.unwrap();
            let clients = clients.lock().await;
            for (_, client) in clients.iter() {
                    let _ = client.send(line.clone());
            }
        }
    });


    let _ = tokio::try_join!(write_task, read_task, server_write_task);


}
