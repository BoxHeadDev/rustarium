use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let stream = TcpStream::connect(addr).await?;
    let (reader, writer) = stream.into_split();

    let mut stdin = BufReader::new(io::stdin()).lines();
    let mut socket_reader = BufReader::new(reader).lines();
    let mut socket_writer = writer;

    // Spawn task to read messages from server
    tokio::spawn(async move {
        while let Ok(Some(line)) = socket_reader.next_line().await {
            println!("{}", line);
        }
    });

    // Read from stdin and send to server
    while let Ok(Some(line)) = stdin.next_line().await {
        socket_writer.write_all(line.as_bytes()).await?;
        socket_writer.write_all(b"\n").await?;
    }

    Ok(())
}

