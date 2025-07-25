use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;

fn main() -> io::Result<()> {
    // Connect to the server
    let mut stream = TcpStream::connect("127.0.0.1:9000")?;
    println!("Connected to chat server.");

    // Prompt for username
    print!("Enter your name: ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    stream.write_all(name.as_bytes())?; // Send username

    let mut stream_clone = stream.try_clone()?;

    // Thread to listen for messages from the server
    thread::spawn(move || {
        let reader = BufReader::new(&mut stream_clone);
        for line in reader.lines() {
            match line {
                Ok(msg) => println!("\r{}\n> ", msg),
                Err(_) => {
                    eprintln!("Disconnected from server.");
                    break;
                }
            }
        }
    });

    // Main thread: user input and send to server
    let stdin = io::stdin();
    print!("> ");
    io::stdout().flush()?;

    for line in stdin.lock().lines() {
        let msg = line?;
        if msg.trim() == "/quit" {
            break;
        }
        stream.write_all(msg.as_bytes())?;
        stream.write_all(b"\n")?;
        print!("> ");
        io::stdout().flush()?;
    }

    println!("Disconnected.");
    Ok(())
}
