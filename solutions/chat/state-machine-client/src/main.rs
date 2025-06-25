use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;

enum ClientState {
    Disconnected,
    Connecting,
    Connected,
    Quitting,
}

fn main() {
    let mut state = ClientState::Disconnected;
    let mut username = String::new();

    loop {
        match state {
            ClientState::Disconnected => {
                println!("Enter server address (e.g., 127.0.0.1:9000):");
                let mut address = String::new();
                io::stdin().read_line(&mut address).unwrap();

                match TcpStream::connect(address.trim()) {
                    Ok(mut stream) => {
                        println!("Connected to server.");
                        state = ClientState::Connecting;

                        // Prompt for name
                        print!("Enter your username: ");
                        io::stdout().flush().unwrap();
                        username.clear();
                        io::stdin().read_line(&mut username).unwrap();
                        stream.write_all(username.as_bytes()).unwrap();

                        let mut reader_stream = stream.try_clone().unwrap();

                        // Spawn receiver thread
                        thread::spawn(move || {
                            let reader = BufReader::new(&mut reader_stream);
                            for line in reader.lines() {
                                match line {
                                    Ok(msg) => println!("\r{}\n> ", msg),
                                    Err(_) => {
                                        println!("❌ Disconnected from server.");
                                        break;
                                    }
                                }
                            }
                        });

                        state = ClientState::Connected;

                        // Sender loop (main thread)
                        print!("> ");
                        io::stdout().flush().unwrap();
                        let stdin = io::stdin();
                        for line in stdin.lock().lines() {
                            let msg = line.unwrap();
                            if msg.trim() == "/quit" {
                                state = ClientState::Quitting;
                                break;
                            }

                            stream.write_all(msg.as_bytes()).unwrap();
                            stream.write_all(b"\n").unwrap();
                            print!("> ");
                            io::stdout().flush().unwrap();
                        }
                    }
                    Err(e) => {
                        println!("Failed to connect: {}", e);
                        state = ClientState::Disconnected;
                    }
                }
            }

            ClientState::Connected => {
                // This state is handled within the sender loop.
                // If we fall through to here, we quit.
                state = ClientState::Quitting;
            }

            ClientState::Quitting => {
                println!("👋 Quitting...");
                break;
            }

            _ => break,
        }
    }
}
