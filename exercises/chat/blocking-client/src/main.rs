use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;

fn main() -> io::Result<()> {
    // Connect to server
    let mut stream = TcpStream::connect("127.0.0.1:9000")?;
    println!("Connected to chat server.");

    // Prompt for username
    print!("Enter your name: ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    stream.write_all(name.as_bytes())?;

    // Clone stream for reading
    let read_stream = stream.try_clone()?;

    // Thread to handle incoming messages
    thread::spawn(move || {
        let reader = BufReader::new(read_stream);
        for line in reader.lines() {
            match line {
                Ok(msg) => println!("\n{}", msg),
                Err(_) => {
                    println!("\nDisconnected from server.");
                    break;
                }
            }
        }
    });

    // Main thread handles user input
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let msg = line?;
        stream.write_all(msg.as_bytes())?;
        stream.write_all(b"\n")?;
    }

    Ok(())
}
