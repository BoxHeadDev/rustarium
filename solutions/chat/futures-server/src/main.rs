use async_std::{
    net::{TcpListener, TcpStream},
    prelude::*,
    task,
};
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type ClientMap = Arc<Mutex<HashMap<String, TcpStream>>>;

async fn handle_client(
    name: String,
    mut stream: TcpStream,
    clients: ClientMap,
) -> async_std::io::Result<()> {
    let mut buf = vec![0u8; 1024];

    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break; // client disconnected
        }

        let msg = String::from_utf8_lossy(&buf[..n]).trim().to_string();
        println!("{}: {}", name, msg);

        let mut to_remove = vec![];

        // Broadcast to all other clients
        for (other_name, other_stream) in clients.lock().unwrap().iter_mut() {
            if other_name != &name {
                if let Err(e) = other_stream.write_all(format!("[{}]: {}\n", name, msg).as_bytes()).await {
                    eprintln!("Failed to write to {}: {}", other_name, e);
                    to_remove.push(other_name.clone());
                }
            }
        }

        // Remove any broken clients
        for name in to_remove {
            clients.lock().unwrap().remove(&name);
        }
    }

    println!("Client {} disconnected", name);
    clients.lock().unwrap().remove(&name);
    Ok(())
}

fn main() -> async_std::io::Result<()> {
    task::block_on(async {
        let listener = TcpListener::bind("127.0.0.1:9000").await?;
        let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));
        println!("Futures chat server running on 127.0.0.1:9000");

        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            match stream {
                Ok(mut stream) => {
                    let mut name_buf = String::new();
                    let mut reader = async_std::io::BufReader::new(stream.clone());
                    reader.read_line(&mut name_buf).await?;
                    let name = name_buf.trim().to_string();

                    println!("New client: {}", name);
                    clients.lock().unwrap().insert(name.clone(), stream.clone());

                    let client_map = clients.clone();
                    task::spawn(handle_client(name, stream, client_map));
                }
                Err(e) => eprintln!("Failed to accept connection: {}", e),
            }
        }

        Ok(())
    })
}

