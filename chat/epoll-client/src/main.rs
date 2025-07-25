use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::os::unix::io::{AsRawFd, RawFd};

#[allow(unused_macros)]
macro_rules! syscall {
    ($fn:ident($($arg:expr),* $(,)?)) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

fn main() -> io::Result<()> {
    let mut socket = TcpStream::connect("127.0.0.1:9000")?;
    socket.set_nonblocking(true)?;
    let socket_fd = socket.as_raw_fd();

    let stdin_fd = io::stdin().as_raw_fd();
    set_fd_nonblocking(stdin_fd)?;

    let epoll_fd = syscall!(epoll_create1(0))?;

    let mut stdin_event = libc::epoll_event {
        events: libc::EPOLLIN as u32,
        u64: 1,
    };
    let mut socket_event = libc::epoll_event {
        events: libc::EPOLLIN as u32,
        u64: 2,
    };

    syscall!(epoll_ctl(
        epoll_fd,
        libc::EPOLL_CTL_ADD,
        stdin_fd,
        &mut stdin_event
    ))?;
    syscall!(epoll_ctl(
        epoll_fd,
        libc::EPOLL_CTL_ADD,
        socket_fd,
        &mut socket_event
    ))?;

    let mut events = vec![libc::epoll_event { events: 0, u64: 0 }; 2];
    let mut stdin_buf = [0u8; 1024];
    let mut socket_buf = [0u8; 1024];

    println!("Connected. Type messages to send:");

    loop {
        let nfds = syscall!(epoll_wait(
            epoll_fd,
            events.as_mut_ptr(),
            events.len() as i32,
            1000,
        ))?;
        unsafe { events.set_len(nfds as usize) }

        for ev in &events {
            match ev.u64 {
                1 => {
                    // stdin readable
                    match io::stdin().read(&mut stdin_buf) {
                        Ok(0) => {
                            println!("stdin closed");
                            return Ok(());
                        }
                        Ok(n) => {
                            let _ = socket.write_all(&stdin_buf[..n]);
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                        Err(e) => return Err(e),
                    }
                }
                2 => {
                    // socket readable
                    match socket.read(&mut socket_buf) {
                        Ok(0) => {
                            println!("server disconnected");
                            return Ok(());
                        }
                        Ok(n) => {
                            print!("{}", String::from_utf8_lossy(&socket_buf[..n]));
                            io::stdout().flush()?;
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                        Err(e) => return Err(e),
                    }
                }
                _ => {}
            }
        }
    }
}

fn set_fd_nonblocking(fd: RawFd) -> io::Result<()> {
    let flags = syscall!(fcntl(fd, libc::F_GETFL))?;
    syscall!(fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK))?;
    Ok(())
}
