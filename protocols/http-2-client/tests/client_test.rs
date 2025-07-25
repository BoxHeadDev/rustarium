use std::{net::TcpListener, thread, time::Duration};

use toy_http2::{client, handle_connection};

fn start_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); // random port
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                toy_http2::handle_connection(&mut stream);
            }
        }
    });

    port
}

#[test]
fn test_client_receives_headers_and_body() {
    let port = start_server();
    std::thread::sleep(Duration::from_millis(100));

    let response = client::send_http2_request("127.0.0.1", port, "/").unwrap();

    assert_eq!(response.headers.get(":status").unwrap(), "200");
    assert_eq!(response.headers.get("content-type").unwrap(), "text/plain");
    assert_eq!(response.body, "Hello from toy HTTP/2 server!");
}
