use nix::sys::epoll::*;
use nix::unistd::{close, read, write};
use std::collections::HashMap;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd, RawFd};
use std::str;

fn main() -> io::Result<()> {
    // Create TCP listener
    let listener = TcpListener::bind("127.0.0.1:9000")?;
    listener.set_nonblocking(true)?;
    println!("Epoll chat server listening on 127.0.0.1:9000");

    let epoll_fd = epoll_create().expect("Failed to create epoll");
    let listener_fd = listener.as_raw_fd();

    let mut clients: HashMap<RawFd, TcpStream> = HashMap::new();

    let event = EpollEvent::new(EpollFlags::EPOLLIN, listener_fd as u64);
    epoll_ctl(epoll_fd, EpollOp::EpollCtlAdd, listener_fd, &event)
        .expect("Failed to add listener to epoll");

    let mut events = vec![EpollEvent::empty(); 1024];

    loop {
        let num_events = epoll_wait(epoll_fd, &mut events, -1).expect("Epoll wait failed");

        for ev in events.iter().take(num_events) {
            let fd = ev.data() as RawFd;

            if fd == listener_fd {
                // Accept new client
                if let Ok((stream, addr)) = listener.accept() {
                    println!("New client: {}", addr);
                    stream.set_nonblocking(true)?;
                    let fd = stream.as_raw_fd();
                    epoll_ctl(
                        epoll_fd,
                        EpollOp::EpollCtlAdd,
                        fd,
                        &EpollEvent::new(EpollFlags::EPOLLIN, fd as u64),
                    )
                    .expect("Failed to add client to epoll");
                    clients.insert(fd, stream);
                }
            } else {
                // Read from client
                let mut buf = [0u8; 512];
                match read(fd, &mut buf) {
                    Ok(0) => {
                        // Connection closed
                        println!("Client disconnected: {}", fd);
                        epoll_ctl(epoll_fd, EpollOp::EpollCtlDel, fd, None).ok();
                        clients.remove(&fd);
                        close(fd).ok();
                    }
                    Ok(n) => {
                        let msg = match str::from_utf8(&buf[..n]) {
                            Ok(s) => s.trim(),
                            Err(_) => continue,
                        };
                        println!("Received from {}: {}", fd, msg);

                        // Broadcast to all clients
                        for (&other_fd, stream) in clients.iter_mut() {
                            if other_fd != fd {
                                let _ = write(
                                    stream.as_raw_fd(),
                                    format!("Client {} says: {}\n", fd, msg).as_bytes(),
                                );
                            }
                        }
                    }
                    Err(_) => {
                        // Ignore read errors
                        continue;
                    }
                }
            }
        }
    }
}
