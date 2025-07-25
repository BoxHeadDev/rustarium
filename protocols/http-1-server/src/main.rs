use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    time::Duration,
};

use mime_guess::from_path;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Listening on http://127.0.0.1:8080");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            std::thread::spawn(move || handle_connection(&mut stream));
        }
    }
    Ok(())
}

fn handle_connection(stream: &mut TcpStream) {
    let peer = stream.peer_addr().unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    loop {
        let mut headers = Vec::new();
        let mut line = String::new();

        while reader.read_line(&mut line).unwrap() > 0 {
            if line == "\r\n" {
                break;
            }
            headers.push(line.trim().to_string());
            line.clear();
        }

        if headers.is_empty() {
            break; // connection closed
        }

        let request_line = &headers[0];
        let mut keep_alive = true;
        let mut is_chunked = false;
        let mut content_length = 0;

        for h in &headers[1..] {
            if h.to_lowercase().starts_with("connection:") && h.to_lowercase().contains("close") {
                keep_alive = false;
            }
            if h.to_lowercase().starts_with("transfer-encoding:")
                && h.to_lowercase().contains("chunked")
            {
                is_chunked = true;
            }
            if h.to_lowercase().starts_with("content-length:") {
                if let Some(len) = h.split(':').nth(1).map(|s| s.trim().parse::<usize>()) {
                    if let Ok(l) = len {
                        content_length = l;
                    }
                }
            }
        }

        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("/");
        let _version = parts.next().unwrap_or("HTTP/1.1");

        let decoded_path = percent_encoding::percent_decode_str(path)
            .decode_utf8_lossy()
            .replace("..", "");

        let file_path = format!(
            "static{}",
            if decoded_path == "/" {
                "/index.html"
            } else {
                &decoded_path
            }
        );
        let path = Path::new(&file_path);

        let mut body = Vec::new();
        if method == "POST" || method == "PUT" {
            if is_chunked {
                loop {
                    let mut chunk_size_line = String::new();
                    reader.read_line(&mut chunk_size_line).unwrap();
                    let size = usize::from_str_radix(chunk_size_line.trim(), 16).unwrap_or(0);
                    if size == 0 {
                        break;
                    }
                    let mut chunk = vec![0; size];
                    reader.read_exact(&mut chunk).unwrap();
                    body.extend_from_slice(&chunk);
                    let _ = reader.read_line(&mut String::new()); // skip CRLF
                }
            } else if content_length > 0 {
                let mut buf = vec![0; content_length];
                reader.read_exact(&mut buf).unwrap();
                body.extend(buf);
            }
        }

        let response = if path.exists() && path.is_file() {
            let contents = fs::read(&path).unwrap_or_else(|_| b"Internal Server Error".to_vec());
            let mime = from_path(&path).first_or_octet_stream();
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: {}\r\n\r\n",
                contents.len(),
                mime,
                if keep_alive { "keep-alive" } else { "close" }
            )
            .into_bytes()
                .into_iter()
                .chain(contents.into_iter())
                .collect::<Vec<u8>>()
        } else {
            let body = b"404 Not Found".to_vec();
            format!(
                "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nConnection: {}\r\n\r\n",
                body.len(),
                if keep_alive { "keep-alive" } else { "close" }
            )
            .into_bytes()
            .into_iter()
            .chain(body.into_iter())
            .collect::<Vec<u8>>()
        };

        stream.write_all(&response).unwrap();
        stream.flush().unwrap();

        if !keep_alive {
            break;
        }

        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
    }

    println!("Connection closed: {}", peer);
}
