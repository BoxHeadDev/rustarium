use nix::sys::epoll::*;
use nix::unistd::{read, write};
use std::io::{self, Write};
use std::net::TcpStream;
use std::os::unix::io::{AsRawFd, RawFd};
use std::str;

fn main() -> io::Result<()> {
    // Connect to the server
    let mut stream = TcpStream::connect("127.0.0.1:9000")?;
    stream.set_nonblocking(true)?;
    println!("Connected to server. Type messages and press Enter.");

    let sock_fd = stream.as_raw_fd();
    let stdin_fd = 0; // STDIN_FILENO

    let epoll_fd = epoll_create().expect("Failed to create epoll");

    // Add socket to epoll
    let sock_event = EpollEvent::new(EpollFlags::EPOLLIN, sock_fd as u64);
    epoll_ctl(epoll_fd, EpollOp::EpollCtlAdd, sock_fd, &sock_event).expect("Failed to add socket");

    // Add stdin to epoll
    let stdin_event = EpollEvent::new(EpollFlags::EPOLLIN, stdin_fd as u64);
    epoll_ctl(epoll_fd, EpollOp::EpollCtlAdd, stdin_fd, &stdin_event).expect("Failed to add stdin");

    let mut events = vec![EpollEvent::empty(); 16];

    loop {
        let nfds = epoll_wait(epoll_fd, &mut events, -1).expect("epoll_wait failed");

        for ev in events.iter().take(nfds) {
            match ev.data() as RawFd {
                fd if fd == sock_fd => {
                    // Incoming message from server
                    let mut buf = [0u8; 512];
                    match read(sock_fd, &mut buf) {
                        Ok(0) => {
                            println!("Server closed connection.");
                            return Ok(());
                        }
                        Ok(n) => {
                            if let Ok(text) = str::from_utf8(&buf[..n]) {
                                print!("\r{}\n> ", text.trim_end());
                                io::stdout().flush().ok();
                            }
                        }
                        Err(_) => {}
                    }
                }

                fd if fd == stdin_fd => {
                    // User input
                    let mut buf = String::new();
                    io::stdin().read_line(&mut buf)?;
                    let msg = buf.trim_end();
                    if msg == "/quit" {
                        println!("Quitting...");
                        return Ok(());
                    }
                    write(sock_fd, msg.as_bytes()).ok();
                    write(sock_fd, b"\n").ok();
                }

                _ => {}
            }
        }

        print!("> ");
        io::stdout().flush().ok();
    }
}
