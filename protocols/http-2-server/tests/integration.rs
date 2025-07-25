use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use toy_http2::{handle_connection, send_frame, HEADERS, DATA}; // make functions pub in main.rs

fn start_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); // bind to random port
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                handle_connection(&mut stream);
            }
        }
    });

    port
}

fn send_test_headers(stream: &mut TcpStream, stream_id: u32) {
    // Very basic payload for fake HPACK-encoded headers
    let payload = b":method:GET\0:path:/\0";
    send_frame(stream, HEADERS, 0x4, stream_id, payload);
}

#[test]
fn test_http2_response_headers_and_data() {
    let port = start_server();
    thread::sleep(Duration::from_millis(100)); // wait for server

    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();

    send_test_headers(&mut stream, 1);

    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).unwrap();
    assert!(n > 0);

    let mut pos = 0;
    while pos + 9 <= n {
        let len = ((buf[pos] as usize) << 16) | ((buf[pos + 1] as usize) << 8) | (buf[pos + 2] as usize);
        let frame_type = buf[pos + 3];
        let flags = buf[pos + 4];
        let stream_id = u32::from_be_bytes([buf[pos + 5], buf[pos + 6], buf[pos + 7], buf[pos + 8]]) & 0x7FFFFFFF;

        let payload = &buf[pos + 9..pos + 9 + len];

        match frame_type {
            HEADERS => {
                let headers = String::from_utf8_lossy(payload);
                assert!(headers.contains(":status:200"));
            }
            DATA => {
                let body = String::from_utf8_lossy(payload);
                assert!(body.contains("Hello from toy HTTP/2 server!"));
            }
            _ => panic!("Unexpected frame type: {}", frame_type),
        }

        pos +=

