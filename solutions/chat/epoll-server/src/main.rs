use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd, RawFd};

#[allow(unused_macros)]
macro_rules! syscall {
    ($fn:ident($($arg:expr),* $(,)?)) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

#[derive(Debug)]
pub struct RequestContext {
    pub stream: TcpStream,
    pub read_buf: Vec<u8>,
    pub write_buf: Vec<u8>,
}

impl RequestContext {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            read_buf: Vec::new(),
            write_buf: Vec::new(),
        }
    }

    fn read_cb(
        &mut self,
        key: u64,
        epoll_fd: RawFd,
        clients: &mut HashMap<u64, RequestContext>,
    ) -> io::Result<()> {
        let mut buf = [0u8; 1024];
        match self.stream.read(&mut buf) {
            Ok(0) => {
                println!("Client {} disconnected", key);
                return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "closed"));
            }
            Ok(n) => {
                let msg = &buf[..n];
                println!("Received from {}: {:?}", key, String::from_utf8_lossy(msg));

                // Queue message to all clients
                for (&k, client) in clients.iter_mut() {
                    if k != key {
                        client.write_buf.extend_from_slice(msg);
                        modify_interest(
                            epoll_fd,
                            client.stream.as_raw_fd(),
                            listener_write_event(k),
                        )?;
                    }
                }

                // Re-arm read
                modify_interest(epoll_fd, self.stream.as_raw_fd(), listener_read_event(key))?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        }
        Ok(())
    }

    fn write_cb(&mut self, key: u64, epoll_fd: RawFd) -> io::Result<()> {
        let n = self.stream.write(&self.write_buf)?;
        self.write_buf.drain(..n);

        if self.write_buf.is_empty() {
            modify_interest(epoll_fd, self.stream.as_raw_fd(), listener_read_event(key))?;
        } else {
            modify_interest(epoll_fd, self.stream.as_raw_fd(), listener_write_event(key))?;
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut clients: HashMap<u64, RequestContext> = HashMap::new();
    let mut events: Vec<libc::epoll_event> = Vec::with_capacity(1024);
    let mut key = 100;

    let listener = TcpListener::bind("127.0.0.1:9000")?;
    listener.set_nonblocking(true)?;
    let listener_fd = listener.as_raw_fd();

    let epoll_fd = epoll_create().expect("epoll_create1 failed");
    add_interest(epoll_fd, listener_fd, listener_read_event(key))?;

    loop {
        events.clear();
        let res = syscall!(epoll_wait(
            epoll_fd,
            events.as_mut_ptr() as *mut libc::epoll_event,
            1024,
            1000,
        ))?;
        unsafe { events.set_len(res as usize) }

        for ev in &events {
            match ev.u64 {
                100 => {
                    match listener.accept() {
                        Ok((stream, addr)) => {
                            stream.set_nonblocking(true)?;
                            println!("New client: {}", addr);
                            key += 1;
                            add_interest(epoll_fd, stream.as_raw_fd(), listener_read_event(key))?;
                            clients.insert(key, RequestContext::new(stream));
                        }
                        Err(e) => eprintln!("Accept failed: {}", e),
                    }
                    modify_interest(epoll_fd, listener_fd, listener_read_event(100))?;
                }
                client_key => {
                    let mut to_remove = None;
                    if let Some(client) = clients.get_mut(&client_key) {
                        let events = ev.events;
                        let fd = client.stream.as_raw_fd();
                        let result = if events & libc::EPOLLIN as u32 != 0 {
                            client.read_cb(client_key, epoll_fd, &mut clients)
                        } else if events & libc::EPOLLOUT as u32 != 0 {
                            client.write_cb(client_key, epoll_fd)
                        } else {
                            Ok(())
                        };

                        if result.is_err() {
                            to_remove = Some(client_key);
                            remove_interest(epoll_fd, fd)?;
                            close(fd);
                        }
                    }
                    if let Some(k) = to_remove {
                        clients.remove(&k);
                    }
                }
            }
        }
    }
}

// Epoll helpers
fn epoll_create() -> io::Result<RawFd> {
    syscall!(epoll_create1(0))
}

fn listener_read_event(key: u64) -> libc::epoll_event {
    libc::epoll_event {
        events: (libc::EPOLLIN | libc::EPOLLONESHOT) as u32,
        u64: key,
    }
}

fn listener_write_event(key: u64) -> libc::epoll_event {
    libc::epoll_event {
        events: (libc::EPOLLOUT | libc::EPOLLONESHOT) as u32,
        u64: key,
    }
}

fn close(fd: RawFd) {
    let _ = syscall!(close(fd));
}

fn add_interest(epoll_fd: RawFd, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
    syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event))?;
    Ok(())
}

fn modify_interest(epoll_fd: RawFd, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
    syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_MOD, fd, &mut event))?;
    Ok(())
}

fn remove_interest(epoll_fd: RawFd, fd: RawFd) -> io::Result<()> {
    syscall!(epoll_ctl(
        epoll_fd,
        libc::EPOLL_CTL_DEL,
        fd,
        std::ptr::null_mut()
    ))?;
    Ok(())
}
