use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type ClientMap = Arc<Mutex<HashMap<String, TcpStream>>>;

struct ChatServer {
    clients: ClientMap,
    on_connect: Arc<Box<dyn Fn(&str) + Send + Sync>>,
    on_message: Arc<Box<dyn Fn(&str, &str) + Send + Sync>>,
    on_disconnect: Arc<Box<dyn Fn(&str) + Send + Sync>>,
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
            on_connect: Arc::new(Box::new(on_connect)),
            on_message: Arc::new(Box::new(on_message)),
            on_disconnect: Arc::new(Box::new(on_disconnect)),
        }
    }

    fn start(&self, addr: &str) {
        let listener = TcpListener::bind(addr).expect("Failed to bind address");
        println!("Chat server running on {}", addr);

        for stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut name = String::new();
            if reader.read_line(&mut name).is_err() {
                continue;
            }
            let name = name.trim().to_string();

            (self.on_connect)(&name);

            {
                let mut clients = self.clients.lock().unwrap();
                clients.insert(name.clone(), stream.try_clone().unwrap());
            }

            let clients = Arc::clone(&self.clients);
            let on_message = Arc::clone(&self.on_message);
            let on_disconnect = Arc::clone(&self.on_disconnect);
            let name_clone = name.clone();

            thread::spawn(move || {
                for msg in reader.lines().map_while(Result::ok) {
                    (on_message)(&name_clone, &msg);

                    let clients_lock = clients.lock().unwrap();
                    for (other_name, client_stream) in clients_lock.iter() {
                        if other_name != &name_clone {
                            if let Ok(mut other_stream) = client_stream.try_clone() {
                                let _ = writeln!(other_stream, "[{}]: {}", name_clone, msg);
                            }
                        }
                    }
                }

                (on_disconnect)(&name_clone);
                clients.lock().unwrap().remove(&name_clone);
            });
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
