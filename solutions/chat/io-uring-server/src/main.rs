use std::collections::VecDeque;
use std::net::TcpListener;
use std::os::unix::io::{AsRawFd, RawFd};
use std::{io, ptr};

use io_uring::{IoUring, SubmissionQueue, opcode, squeue, types};
use slab::Slab;

#[derive(Clone, Debug)]
enum Token {
    Accept,
    Poll {
        fd: RawFd,
    },
    Read {
        fd: RawFd,
        buf_index: usize,
    },
    Write {
        fd: RawFd,
        buf_index: usize,
        offset: usize,
        len: usize,
    },
}

pub struct AcceptCount {
    entry: squeue::Entry,
    count: usize,
}

impl AcceptCount {
    fn new(fd: RawFd, token: usize, count: usize) -> AcceptCount {
        AcceptCount {
            entry: opcode::Accept::new(types::Fd(fd), ptr::null_mut(), ptr::null_mut())
                .build()
                .user_data(token as _),
            count,
        }
    }

    pub fn push_to(&mut self, sq: &mut SubmissionQueue<'_>) {
        while self.count > 0 {
            unsafe {
                match sq.push(&self.entry) {
                    Ok(_) => self.count -= 1,
                    Err(_) => break,
                }
            }
        }

        sq.sync();
    }
}

fn main() -> anyhow::Result<()> {
    let mut ring = IoUring::new(256)?;
    let listener = TcpListener::bind(("127.0.0.1", 3456))?;

    let mut backlog = VecDeque::new();
    let mut bufpool = Vec::with_capacity(64);
    let mut buf_alloc = Slab::with_capacity(64);
    let mut token_alloc = Slab::with_capacity(64);
    let mut clients = std::collections::HashSet::new(); // track connected clients

    println!("Listening on {}", listener.local_addr()?);

    let (submitter, mut sq, mut cq) = ring.split();
    let mut accept = AcceptCount::new(listener.as_raw_fd(), token_alloc.insert(Token::Accept), 3);
    accept.push_to(&mut sq);

    loop {
        match submitter.submit_and_wait(1) {
            Ok(_) => (),
            Err(ref err) if err.raw_os_error() == Some(libc::EBUSY) => (),
            Err(err) => return Err(err.into()),
        }
        cq.sync();

        // Clean backlog
        while let Some(sqe) = backlog.pop_front() {
            if sq.is_full() {
                match submitter.submit() {
                    Ok(_) => (),
                    Err(ref err) if err.raw_os_error() == Some(libc::EBUSY) => {
                        backlog.push_front(sqe);
                        break;
                    }
                    Err(err) => return Err(err.into()),
                }
            }
            unsafe {
                let _ = sq.push(&sqe);
            }
        }

        accept.push_to(&mut sq);

        for cqe in &mut cq {
            let ret = cqe.result();
            let token_index = cqe.user_data() as usize;

            if ret < 0 {
                eprintln!(
                    "token {:?} error: {:?}",
                    token_alloc.get(token_index),
                    io::Error::from_raw_os_error(-ret)
                );
                token_alloc.remove(token_index);
                continue;
            }

            let token = &mut token_alloc[token_index];
            match token.clone() {
                Token::Accept => {
                    println!("Accepted new connection");
                    accept.count += 1;

                    let fd = ret;
                    clients.insert(fd);

                    let poll_token = token_alloc.insert(Token::Poll { fd });
                    let poll_e = opcode::PollAdd::new(types::Fd(fd), libc::POLLIN as _)
                        .build()
                        .user_data(poll_token as _);

                    unsafe {
                        if sq.push(&poll_e).is_err() {
                            backlog.push_back(poll_e);
                        }
                    }
                }
                Token::Poll { fd } => {
                    let (buf_index, buf) = match bufpool.pop() {
                        Some(idx) => (idx, &mut buf_alloc[idx]),
                        None => {
                            let buf = vec![0u8; 2048].into_boxed_slice();
                            let entry = buf_alloc.vacant_entry();
                            let idx = entry.key();
                            (idx, entry.insert(buf))
                        }
                    };

                    *token = Token::Read { fd, buf_index };

                    let read_e = opcode::Recv::new(types::Fd(fd), buf.as_mut_ptr(), buf.len() as _)
                        .build()
                        .user_data(token_index as _);

                    unsafe {
                        if sq.push(&read_e).is_err() {
                            backlog.push_back(read_e);
                        }
                    }
                }

                Token::Read { fd, buf_index } => {
                    if ret == 0 {
                        println!("Client disconnected: {}", fd);
                        clients.remove(&fd);
                        bufpool.push(buf_index);
                        token_alloc.remove(token_index);
                        unsafe {
                            libc::close(fd);
                        }
                    } else {
                        let len = ret as usize;

                        // Clone message out of the slab first
                        let message: Vec<u8> = buf_alloc[buf_index][..len].to_vec();

                        // Safe to reuse the buffer now
                        bufpool.push(buf_index);

                        for &other_fd in &clients {
                            if other_fd == fd {
                                continue;
                            }

                            let msg_buf: Box<[u8]> = message.clone().into_boxed_slice(); // clone per client
                            let new_buf_index = buf_alloc.insert(msg_buf);

                            let write_token = Token::Write {
                                fd: other_fd,
                                buf_index: new_buf_index,
                                offset: 0,
                                len,
                            };
                            let write_token_index = token_alloc.vacant_entry().key();
                            token_alloc.insert(write_token);

                            let write_e = opcode::Send::new(
                                types::Fd(other_fd),
                                buf_alloc[new_buf_index].as_ptr(),
                                len as _,
                            )
                            .build()
                            .user_data(write_token_index as _);

                            unsafe {
                                if sq.push(&write_e).is_err() {
                                    backlog.push_back(write_e);
                                }
                            }
                        }

                        // Re-arm polling for sender
                        token_alloc[token_index] = Token::Poll { fd };

                        let poll_e = opcode::PollAdd::new(types::Fd(fd), libc::POLLIN as _)
                            .build()
                            .user_data(token_index as _);

                        unsafe {
                            if sq.push(&poll_e).is_err() {
                                backlog.push_back(poll_e);
                            }
                        }
                    }
                }

                Token::Write {
                    fd,
                    buf_index,
                    offset,
                    len,
                } => {
                    let written = ret as usize;
                    if offset + written >= len {
                        bufpool.push(buf_index);
                        *token = Token::Poll { fd };

                        let poll_e = opcode::PollAdd::new(types::Fd(fd), libc::POLLIN as _)
                            .build()
                            .user_data(token_index as _);

                        unsafe {
                            if sq.push(&poll_e).is_err() {
                                backlog.push_back(poll_e);
                            }
                        }
                    } else {
                        let new_offset = offset + written;
                        let buf = &buf_alloc[buf_index][new_offset..];

                        *token = Token::Write {
                            fd,
                            buf_index,
                            offset: new_offset,
                            len,
                        };

                        let write_e =
                            opcode::Write::new(types::Fd(fd), buf.as_ptr(), buf.len() as _)
                                .build()
                                .user_data(token_index as _);

                        unsafe {
                            if sq.push(&write_e).is_err() {
                                backlog.push_back(write_e);
                            }
                        }
                    }
                }
            }
        }
    }
}
