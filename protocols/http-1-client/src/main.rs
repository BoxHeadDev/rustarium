use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
};

fn main() {
    let host = "example.com";
    let port = 80;
    let path = "/";

    let mut stream = TcpStream::connect((host, port)).expect("Failed to connect");

    // Send request
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, host
    );
    stream.write_all(request.as_bytes()).unwrap();

    // Read response
    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader.read_line(&mut status_line).unwrap();
    println!("Status: {}", status_line.trim_end());

    let mut headers = Vec::new();
    let mut content_length = None;
    let mut is_chunked = false;

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if line == "\r\n" {
            break;
        }
        let lower = line.to_lowercase();
        if lower.starts_with("content-length:") {
            content_length = lower
                .splitn(2, ':')
                .nth(1)
                .and_then(|v| v.trim().parse::<usize>().ok());
        } else if lower.starts_with("transfer-encoding:") && lower.contains("chunked") {
            is_chunked = true;
        }
        headers.push(line.trim().to_string());
    }

    println!("Headers:\n{}", headers.join("\n"));
    println!("Body:");

    if is_chunked {
        // Read chunked body
        loop {
            let mut size_line = String::new();
            reader.read_line(&mut size_line).unwrap();
            let size = usize::from_str_radix(size_line.trim(), 16).unwrap_or(0);
            if size == 0 {
                break;
            }
            let mut chunk = vec![0; size];
            reader.read_exact(&mut chunk).unwrap();
            println!("{}", String::from_utf8_lossy(&chunk));
            let _ = reader.read_line(&mut String::new()); // Read trailing CRLF
        }
    } else if let Some(len) = content_length {
        let mut body = vec![0; len];
        reader.read_exact(&mut body).unwrap();
        println!("{}", String::from_utf8_lossy(&body));
    } else {
        let mut body = String::new();
        reader.read_to_string(&mut body).unwrap();
        println!("{}", body);
    }
}
