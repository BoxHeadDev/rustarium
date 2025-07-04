use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;

struct ChatClient {
    on_connect: Arc<Box<dyn Fn() + Send + Sync>>,
    on_message: Arc<Box<dyn Fn(&str) + Send + Sync>>,
    on_disconnect: Arc<Box<dyn Fn() + Send + Sync>>,
}

impl ChatClient {
    fn new<F1, F2, F3>(on_connect: F1, on_message: F2, on_disconnect: F3) -> Self
    where
        F1: Fn() + Send + Sync + 'static,
        F2: Fn(&str) + Send + Sync + 'static,
        F3: Fn() + Send + Sync + 'static,
    {
        ChatClient {
            on_connect: Arc::new(Box::new(on_connect)),
            on_message: Arc::new(Box::new(on_message)),
            on_disconnect: Arc::new(Box::new(on_disconnect)),
        }
    }

    fn run(&self, address: &str, username: &str) -> io::Result<()> {
        let mut stream = TcpStream::connect(address)?;
        (self.on_connect)();

        writeln!(stream, "{}", username)?;

        let stream_clone = stream.try_clone()?;

        // Clone the Arc pointers (cheap, shared ownership)
        let on_message = Arc::clone(&self.on_message);
        let on_disconnect = Arc::clone(&self.on_disconnect);

        thread::spawn(move || {
            let reader = BufReader::new(stream_clone);
            for line in reader.lines() {
                match line {
                    Ok(msg) => (on_message)(&msg),
                    Err(_) => {
                        (on_disconnect)();
                        break;
                    }
                }
            }
        });

        // Main thread sends user input
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let msg = line?;
            writeln!(stream, "{}", msg)?;
        }

        Ok(())
    }
}

fn main() {
    let client = ChatClient::new(
        || println!("Connected to server"),
        |msg| println!("Received: {}", msg),
        || println!("Disconnected from server"),
    );

    println!("Enter your username:");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username).unwrap();
    let username = username.trim();

    client.run("127.0.0.1:9000", username).unwrap();
}
