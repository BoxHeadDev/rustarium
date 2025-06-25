use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type ClientMap = Arc<Mutex<HashMap<String, TcpStream>>>;

struct ChatServer {
    clients: ClientMap,
    on_connect: Box<dyn Fn(&str) + Send + Sync>,
    on_message: Box<dyn Fn(&str, &str) + Send + Sync>,
    on_disconnect: Box<dyn Fn(&str) + Send + Sync>,
}

impl ChatServer {
    fn new<F1, F2, F3>(on_connect: F1, on_message: F2, on_disconnect: F3) -> Self
    where
        F1: Fn(&str) + Send + Sync + 'static,
        F2: Fn(&str, &str) + Send + Sync + 'static,
        F3: Fn(&str) + Send + Sync + 'static,
    {
        ChatServer {
            clients: Arc::new(Mutex::new(HashMap::new())),
            on_connect: Box::new(on_connect),
            on_message: Box::new(on_message),
            on_disconnect: Box::new(on_disconnect),
        }
    }

    fn start(&self, addr: &str) {
        let listener = TcpListener::bind(addr).expect("Failed to bind address");
        println!("Chat server running on {}", addr);

        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut name = String::new();
                if reader.read_line(&mut name).is_err() {
                    continue;
                }
                let name = name.trim().to_string();

                (self.on_connect)(&name);

                let mut clients = self.clients.lock().unwrap();
                clients.insert(name.clone(), stream.try_clone().unwrap());
                drop(clients);

                let clients = Arc::clone(&self.clients);
                let on_message = self.on_message.clone();
                let on_disconnect = self.on_disconnect.clone();

                thread::spawn(move || {
                    for line in reader.lines() {
                        let line = match line {
                            Ok(msg) => msg,
                            Err(_) => break,
                        };
                        on_message(&name, &line);

                        let clients_lock = clients.lock().unwrap();
                        for (other_name, client_stream) in clients_lock.iter() {
                            if other_name != &name {
                                let _ = writeln!(
                                    &mut client_stream.try_clone().unwrap(),
                                    "[{}]: {}",
                                    name,
                                    line
                                );
                            }
                        }
                    }

                    on_disconnect(&name);
                    clients.lock().unwrap().remove(&name);
                });
            }
        }
    }
}

fn main() {
    let server = ChatServer::new(
        |name| println!("{} joined", name),
        |name, msg| println!("📨 {} says: {}", name, msg),
        |name| println!("{} left", name),
    );

    server.start("127.0.0.1:9000");
}
