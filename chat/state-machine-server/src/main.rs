use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug)]
enum ClientState {
    AwaitingUsername,
    Connected(String),
    Disconnected,
}

type SharedClients = Arc<Mutex<HashMap<String, TcpStream>>>;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9000").expect("Failed to bind");
    println!("State machine chat server running on 127.0.0.1:9000");

    let clients: SharedClients = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let clients = Arc::clone(&clients);
                thread::spawn(move || handle_client(stream, clients));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

fn handle_client(stream: TcpStream, clients: SharedClients) {
    let mut state = ClientState::AwaitingUsername;
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    loop {
        let mut input = String::new();
        let bytes_read = reader.read_line(&mut input);
        if bytes_read.is_err() || bytes_read.unwrap() == 0 {
            state = ClientState::Disconnected;
        }

        match &mut state {
            ClientState::AwaitingUsername => {
                let name = input.trim().to_string();
                if name.is_empty() {
                    let _ = writeln!(
                        stream.try_clone().unwrap(),
                        "Please enter a valid username:"
                    );
                } else {
                    println!("User '{}' connected", name);
                    clients
                        .lock()
                        .unwrap()
                        .insert(name.clone(), stream.try_clone().unwrap());
                    let _ = writeln!(stream.try_clone().unwrap(), "Welcome, {}!", name);
                    state = ClientState::Connected(name);
                }
            }

            ClientState::Connected(name) => {
                let msg = input.trim();
                if msg == "/quit" {
                    println!("{} disconnected", name);
                    clients.lock().unwrap().remove(name);
                    state = ClientState::Disconnected;
                } else {
                    let full_msg = format!("[{}]: {}\n", name, msg);
                    let client_list = clients.lock().unwrap();
                    for (other_name, other_stream) in client_list.iter() {
                        if other_name != name {
                            let _ = other_stream
                                .try_clone()
                                .unwrap()
                                .write_all(full_msg.as_bytes());
                        }
                    }
                }
            }

            ClientState::Disconnected => {
                println!("Cleaning up client.");
                break;
            }
        }
    }
}
