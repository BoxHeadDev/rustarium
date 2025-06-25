use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type ClientList = Arc<Mutex<HashMap<String, TcpStream>>>;

fn handle_client(mut stream: TcpStream, name: String, clients: ClientList) {
    let reader = BufReader::new(stream.try_clone().expect("Failed to clone stream"));

    for line in reader.lines() {
        let msg = match line {
            Ok(msg) => msg,
            Err(_) => break,
        };

        if msg.starts_with('@') {
            // Private message
            if let Some((target, content)) = msg.split_once(' ') {
                let target_name = target.trim_start_matches('@');
                let clients_lock = clients.lock().unwrap();
                if let Some(target_stream) = clients_lock.get(target_name) {
                    let _ = writeln!(
                        &mut target_stream.try_clone().unwrap(),
                        "[{} -> you]: {}",
                        name,
                        content
                    );
                }
            }
        } else {
            // Broadcast
            let clients_lock = clients.lock().unwrap();
            for (client_name, client_stream) in clients_lock.iter() {
                if client_name != &name {
                    let _ = writeln!(
                        &mut client_stream.try_clone().unwrap(),
                        "[{}]: {}",
                        name,
                        msg
                    );
                }
            }
        }
    }

    // Remove disconnected client
    clients.lock().unwrap().remove(&name);
    println!("Client {} disconnected", name);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9000").expect("Could not bind");
    let clients: ClientList = Arc::new(Mutex::new(HashMap::new()));
    println!("Server listening on port 9000");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut name = String::new();
                let mut reader =
                    BufReader::new(stream.try_clone().expect("Failed to clone stream"));
                let _ = reader.read_line(&mut name);
                name = name.trim().to_string();

                println!("Client '{}' connected", name);
                clients
                    .lock()
                    .unwrap()
                    .insert(name.clone(), stream.try_clone().unwrap());

                let clients_clone = Arc::clone(&clients);
                thread::spawn(move || {
                    handle_client(stream, name, clients_clone);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
