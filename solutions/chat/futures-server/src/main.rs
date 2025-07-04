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

        // Clone the streams while holding the lock
        let client_snapshot = {
            let guard = clients.lock().unwrap();
            guard
                .iter()
                .filter(|(other_name, _)| *other_name != &name)
                .map(|(other_name, s)| (other_name.clone(), s.clone()))
                .collect::<Vec<_>>()
        };

        let mut to_remove = vec![];

        for (other_name, mut other_stream) in client_snapshot {
            if let Err(e) = other_stream
                .write_all(format!("[{}]: {}\n", name, msg).as_bytes())
                .await
            {
                eprintln!("Failed to write to {}: {}", other_name, e);
                to_remove.push(other_name);
            }
        }

        if !to_remove.is_empty() {
            let mut guard = clients.lock().unwrap();
            for name in to_remove {
                guard.remove(&name);
            }
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
                Ok(stream) => {
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
