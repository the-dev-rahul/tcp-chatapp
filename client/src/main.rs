
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let socket = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);

    let write_task = tokio::spawn(async move {
        let mut stdin = BufReader::new(io::stdin());
        let mut line = String::new();
        while let Ok(_message) = stdin.read_line(&mut line).await{
            if writer.write_all(line.as_bytes()).await.is_err(){
                break;
            }
            line.clear();
        }

    });

    let mut line = String::new();
    let read_task = tokio::spawn(async move{

        while let Ok(_message) = reader.read_line(&mut line).await{
            if ! line.is_empty() {
                print!("Received: {}", line);}
            else {
                break;
            }
            line.clear();

        }
    });

    
    let _ = tokio::try_join!(write_task, read_task);
    Ok(())
}
