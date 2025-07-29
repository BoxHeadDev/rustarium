use async_std::io;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task;
use futures::{select, FutureExt, StreamExt};

fn main() {
    task::block_on(async {
        // Prompt for username
        println!("Enter your username:");
        let mut username = String::new();
        io::stdin().read_line(&mut username).await.unwrap();
        let username = username.trim().to_string();

        let stream = TcpStream::connect("127.0.0.1:9000").await.unwrap();
        println!("Connected to server.");
        let (reader, writer) = &mut (&stream, &stream);

        // Send username first
        writer.write_all(format!("{}\n", username).as_bytes()).await.unwrap();

        let stdin = io::BufReader::new(io::stdin());
        let mut stdin_lines = stdin.lines().fuse();

        let socket_reader = io::BufReader::new(reader);
        let mut server_lines = socket_reader.lines().fuse();

        loop {
            select! {
                line = stdin_lines.next().fuse() => match line {
                    Some(Ok(line)) => {
                        writer.write_all(format!("{}\n", line).as_bytes()).await.unwrap();
                    }
                    _ => break,
                },

                msg = server_lines.next().fuse() => match msg {
                    Some(Ok(msg)) => {
                        println!("{}", msg);
                    }
                    Some(Err(e)) => {
                        eprintln!("Error reading from server: {}", e);
                        break;
                    }
                    None => {
                        println!("Server disconnected.");
                        break;
                    }
                },
            }
        }

        println!("Exiting chat.");
    });
}

