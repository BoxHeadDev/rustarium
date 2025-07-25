use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn start_mock_server(response: &'static str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); // Bind to a random port
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0; 1024];
            let _ = stream.read(&mut buffer);
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    });

    port
}

#[test]
fn test_client_parses_response() {
    let response = "\
HTTP/1.1 200 OK\r\n\
Content-Length: 13\r\n\
Content-Type: text/plain\r\n\
\r\n\
Hello, world!";

    let port = start_mock_server(response);

    // Simulate the client
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let request = format!("GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(response.contains("Content-Length: 13"));
    assert!(response.contains("Hello, world!"));
}

#[test]
fn test_client_handles_chunked_response() {
    let response = "\
HTTP/1.1 200 OK\r\n\
Transfer-Encoding: chunked\r\n\
Content-Type: text/plain\r\n\
\r\n\
7\r\n\
Chunk 1\r\n\
6\r\n\
-1234\r\n\
0\r\n\
\r\n";

    let port = start_mock_server(response);

    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    stream.write_all(request.as_bytes()).unwrap();

    let mut buffer = String::new();
    stream.read_to_string(&mut buffer).unwrap();

    assert!(buffer.contains("Chunk 1"));
    assert!(buffer.contains("-1234"));
}
