use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, UnboundedSender},
};

type Tx = UnboundedSender<String>;
type SharedConnections = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    let connections: SharedConnections = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("New Connection: {}", addr);

        let connections = connections.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr, connections).await {
                eprintln!("Error handling {}: {:?}", addr, e);
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    connections: SharedConnections,
) -> anyhow::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    let (tx, mut rx) = mpsc::unbounded_channel();
    connections.lock().unwrap().insert(addr, tx);

    // Task to send messages to this client
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if writer.write_all(msg.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    // Read messages from this client and broadcast to others
    while let Ok(Some(line)) = lines.next_line().await {
        let msg = format!("{}: {}\n", addr, line);
        println!("{}", msg);
        let conns = connections.lock().unwrap();
        for (&peer, tx) in conns.iter() {
            if peer != addr {
                let _ = tx.send(msg.clone());
            }
        }
    }

    // Cleanup
    connections.lock().unwrap().remove(&addr);
    println!("Connection {} closed.", addr);
    write_task.abort();

    Ok(())
}

