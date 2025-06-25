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

        let formatted = format!("[{}]: {}\n", name, msg);

        // Broadcast to other clients
        let clients_lock = clients.lock().unwrap();
        for (other_name, other_stream) in clients_lock.iter() {
            if other_name != &name {
                let _ = other_stream
                    .try_clone()
                    .and_then(|mut s| s.write_all(formatted.as_bytes()));
            }
        }
    }

    println!("{} disconnected", name);
    clients.lock().unwrap().remove(&name);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9000").expect("Failed to bind");
    println!("Threaded chat server running on 127.0.0.1:9000");

    let clients: ClientList = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut name = String::new();
                if reader.read_line(&mut name).is_err() {
                    continue;
                }
                let name = name.trim().to_string();

                println!("{} connected", name);
                clients
                    .lock()
                    .unwrap()
                    .insert(name.clone(), stream.try_clone().unwrap());

                let clients_clone = clients.clone();
                thread::spawn(move || {
                    handle_client(stream, name, clients_clone);
                });
            }
            Err(e) => eprintln!("Connection error: {}", e),
        }
    }
}
