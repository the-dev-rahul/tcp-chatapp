use std::io::{Error, ErrorKind};
use tokio::net::UdpSocket;
use bytes::{BufMut, BytesMut};

#[derive(Debug)]
struct Message {
  sender_id: u32,
  content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Server address
  let server_addr = "127.0.0.1:8080";

  // Create socket
  let socket = UdpSocket::bind(server_addr).await?;
  println!("Server listening on {}", server_addr);

  let mut buf = BytesMut::with_capacity(1024);
  loop {
    // Receive message
    let (len, addr) = socket.recv_from(&mut buf).await?;
    buf.truncate(len); // Clear buffer for next message

    // Decode message
    let message = match serde_json::from_slice(&buf) {
      Ok(msg) => msg,
      Err(err) => {
        eprintln!("Error decoding message: {}", err);
        continue;
      }
    };

    // Validate message format
    if message.sender_id == 0 || message.content.is_empty() {
      eprintln!("Invalid message format!");
      continue;
    }

    println!("Received from {}: {:?}", addr, message);

    // Send acknowledgement (optional, for improved reliability)
    let ack_message = Message { sender_id: 0, content: String::from("ACK") };
    let encoded_ack = serde_json::to_vec(&ack_message)?;
    socket.send_to(&encoded_ack, addr).await?;

    // Clear buffer for next iteration
    buf.clear();
  }
}
