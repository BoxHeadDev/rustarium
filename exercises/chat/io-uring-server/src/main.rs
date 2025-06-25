use std::{
    collections::HashMap,
    net::SocketAddr,
    rc::Rc,
    cell::RefCell,
};
use tokio_uring::net::TcpListener;
use bytes::BytesMut;

type ClientMap = Rc<RefCell<HashMap<SocketAddr, tokio_uring::net::TcpStream>>>;

fn main() {
    tokio_uring::start(async {
        let listener = TcpListener::bind("127.0.0.1:9000").unwrap();
        println!("io_uring chat server running on 127.0.0.1:9000");

        let clients: ClientMap = Rc::new(RefCell::new(HashMap::new()));

        loop {
            let (stream, addr) = listener.accept().await.unwrap();
            println!("Client connected: {}", addr);
            stream.set_nodelay(true).unwrap();

            let client_map = clients.clone();
            client_map.borrow_mut().insert(addr, stream.clone());

            tokio_uring::spawn(handle_client(stream, addr, client_map));
        }
    });
}

async fn handle_client(
    mut stream: tokio_uring::net::TcpStream,
    addr: SocketAddr,
    clients: ClientMap,
) {
    let mut buf = BytesMut::with_capacity(1024);

    loop {
        let (read_result, new_buf) = stream.read(buf).await;
        buf = new_buf;

        let n = match read_result {
            Ok(0) => {
                println!("Client disconnected: {}", addr);
                clients.borrow_mut().remove(&addr);
                return;
            }
            Ok(n) => n,
            Err(_) => {
                eprintln!("Error reading from client: {}", addr);
                clients.borrow_mut().remove(&addr);
                return;
            }
        };

        let msg = String::from_utf8_lossy(&buf[..n]).trim().to_string();
        println!("{}: {}", addr, msg);

        let mut disconnected = Vec::new();

        for (client_addr, client_stream) in clients.borrow().iter() {
            if *client_addr != addr {
                let full_msg = format!("[{}]: {}\n", addr, msg);
                let data = full_msg.as_bytes().to_vec();

                let (res, _) = client_stream.write_all(data).await;
                if res.is_err() {
                    disconnected.push(*client_addr);
                }
            }
        }

        for dc in disconnected {
            clients.borrow_mut().remove(&dc);
            println!("Removed disconnected client: {}", dc);
        }

        buf.clear();
    }
}

