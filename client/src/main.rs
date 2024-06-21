use std::io::{Error, ErrorKind};
use tokio::net::UdpSocket;
use bytes::BytesMut;

#[derive(Debug)]
struct Message {
  sender_id: u32,
  content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Client ID
  let client_id = 1;

  // Server address
  let server_addr = "127.0.0.1:8080";

  // Create socket
  let socket = UdpSocket::bind("0.0.0.0:0").await?;

  let mut buf = BytesMut::with_capacity(1024);

  loop {
    // Get user input
    let mut message = String::new();
    std::io::stdin().read_line(&mut message)?;

    // Create message
    let message = Message { sender_id, content: message.trim().to_string() };

    // Encode message
    let encoded_message = serde_json::to_vec(&message)?;

    // Send message to server
    socket.send_to(&encoded_message, server_addr).await?;

    // Receive acknowledgement (optional, for improved reliability)
    let (len, _) = socket.recv_from(&mut buf).await?;
    buf.truncate(len);

    let ack_message = match serde_json::from_slice(&buf) {
      Ok(msg) => msg,
      Err(_) => {
        eprintln!("Error receiving acknowledgement!");
        continue;
      }
    };

    if ack_message.content != "ACK" {
      eprintln!("Unexpected response from server!");
    }

    // Clear buffer for next iteration
    buf.clear();
  }
}
