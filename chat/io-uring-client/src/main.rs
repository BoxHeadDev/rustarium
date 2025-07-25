use io_uring::{IoUring, opcode, types};
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

const BUF_SIZE: usize = 2048;

fn main() -> anyhow::Result<()> {
    let ring = IoUring::new(256)?;

    let stream = TcpStream::connect("127.0.0.1:3456")?;
    stream.set_nonblocking(true)?;
    let sock_fd = stream.as_raw_fd();

    let stdin_fd = 0; // STDIN_FILENO

    let mut input_buf = vec![0u8; BUF_SIZE];
    let mut socket_buf = vec![0u8; BUF_SIZE];

    let mut ring = ring;
    let (submitter, mut sq, mut cq) = ring.split();

    loop {
        // Queue read from stdin
        let stdin_read =
            opcode::Read::new(types::Fd(stdin_fd), input_buf.as_mut_ptr(), BUF_SIZE as _)
                .build()
                .user_data(1);

        unsafe {
            sq.push(&stdin_read).unwrap();
        }

        // Queue read from socket
        let socket_read =
            opcode::Recv::new(types::Fd(sock_fd), socket_buf.as_mut_ptr(), BUF_SIZE as _)
                .build()
                .user_data(2);

        unsafe {
            sq.push(&socket_read).unwrap();
        }

        submitter.submit_and_wait(1)?;
        cq.sync();

        for cqe in &mut cq {
            let result = cqe.result();
            let user_data = cqe.user_data();

            if result <= 0 {
                println!("Connection closed or error occurred");
                return Ok(());
            }

            if user_data == 1 {
                // stdin input -> send to server
                let n = result as usize;
                let msg = &input_buf[..n];

                let write_e = opcode::Send::new(types::Fd(sock_fd), msg.as_ptr(), msg.len() as _)
                    .build()
                    .user_data(3);

                unsafe {
                    sq.push(&write_e).unwrap();
                }

                submitter.submit()?;
            } else if user_data == 2 {
                // message from server -> print to stdout
                let n = result as usize;
                let msg = &socket_buf[..n];

                if let Ok(s) = std::str::from_utf8(msg) {
                    print!("{}", s);
                } else {
                    println!("Received binary data");
                }
            }
        }
    }
}
