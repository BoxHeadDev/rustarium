use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

fn wait_for_server() {
    for _ in 0..10 {
        if TcpStream::connect("127.0.0.1:8080").is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(200));
    }
    panic!("Server did not start in time");
}

#[test]
fn test_static_file_serving() {
    // Ensure static file exists
    fs::create_dir_all("static").unwrap();
    fs::write("static/test.txt", b"Hello from test!").unwrap();

    // Start server in a separate thread
    thread::spawn(|| {
        let _ = Command::new("cargo")
            .args(&["run"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to run server");
    });

    wait_for_server();

    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    let request = "GET /test.txt HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    stream.write_all(request.as_bytes()).unwrap();

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).unwrap();
    let response = String::from_utf8_lossy(&buffer);

    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(response.contains("Hello from test!"));
}

#[test]
fn test_404_response() {
    wait_for_server();

    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    let request = "GET /nonexistent.html HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    stream.write_all(request.as_bytes()).unwrap();

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).unwrap();
    let response = String::from_utf8_lossy(&buffer);

    assert!(response.contains("HTTP/1.1 404 Not Found"));
}

#[test]
fn test_keep_alive() {
    wait_for_server();

    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    let request = "GET /test.txt HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n";
    stream.write_all(request.as_bytes()).unwrap();

    let mut buffer = [0u8; 1024];
    let size = stream.read(&mut buffer).unwrap();
    let response = String::from_utf8_lossy(&buffer[..size]);

    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(response.contains("Connection: keep-alive"));

    // Send second request over same connection
    stream.write_all(request.as_bytes()).unwrap();
    let size2 = stream.read(&mut buffer).unwrap();
    let response2 = String::from_utf8_lossy(&buffer[..size2]);

    assert!(response2.contains("HTTP/1.1 200 OK"));
}
