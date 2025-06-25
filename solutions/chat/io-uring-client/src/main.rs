use tokio_uring::fs::File;
use tokio_uring::net::TcpStream;
use tokio_uring::start;

use bytes::BytesMut;
use std::io;
use std::rc::Rc;
use std::cell::RefCell;
use futures::StreamExt;

fn main() {
    start(async {
        let stream = TcpStream::connect("127.0.0.1:9000").await.unwrap();
        println!("Connected to server. Type messages and press Enter.");

        let (read_half, write_half) = stream.split();

        let write_half = Rc::new(RefCell::new(write_half));
        let stdin_file = File::open("/dev/stdin").unwrap();
        let stdin = Rc::new(RefCell::new(stdin_file));

        // Task: read from socket
        let socket_task = {
            let mut socket_buf = BytesMut::with_capacity(1024);
            async move {
                let mut socket_read = read_half;
                loop {
                    let (res, buf) = socket_read.read(socket_buf).await;
                    socket_buf = buf;
                    match res {
                        Ok(0) => {
                            println!("Disconnected from server.");
                            break;
                        }
                        Ok(n) => {
                            print!("{}", String::from_utf8_lossy(&socket_buf[..n]));
                            socket_buf.clear();
                        }
                        Err(e) => {
                            eprintln!("Socket read error: {}", e);
                            break;
                        }
                    }
                }
            }
        };

        // Task: read from stdin and send to server
        let stdin_task = {
            let stdin = stdin.clone();
            let write_half = write_half.clone();
            async move {
                loop {
                    let mut buf = BytesMut::with_capacity(1024);
                    let (res, buf) = stdin.borrow_mut().read(buf).await;
                    let buf = &buf[..res.unwrap_or(0)];

                    if buf.is_empty() {
                        break;
                    }

                    let (res, _) = write_half.borrow_mut().write_all(buf.to_vec()).await;
                    if res.is_err() {
                        eprintln!("Error writing to server.");
                        break;
                    }
                }
            }
        };

        tokio_uring::join!(socket_task, stdin_task);
    });
}

