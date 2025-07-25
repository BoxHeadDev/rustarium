use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use bytes::{Buf, BytesMut};

const FRAME_HEADER_LEN: usize = 9;

// Frame types
const DATA: u8 = 0x0;
const HEADERS: u8 = 0x1;

// A toy static HPACK-like header table
fn decode_headers(payload: &[u8]) -> HashMap<String, String> {
    // In a real implementation, parse HPACK. Here, simulate static decode.
    let mut headers = HashMap::new();
    headers.insert(":method".into(), "GET".into());
    headers.insert(":path".into(), "/".into());
    headers.insert(":scheme".into(), "http".into());
    headers.insert("user-agent".into(), "toy-client/0.1".into());
    headers
}

// Encode headers into a fake HPACK format (toy)
fn encode_headers(headers: &HashMap<&str, &str>) -> Vec<u8> {
    // Simulate static HPACK. No compression.
    let mut encoded = Vec::new();
    for (k, v) in headers {
        encoded.extend_from_slice(k.as_bytes());
        encoded.push(b':');
        encoded.extend_from_slice(v.as_bytes());
        encoded.push(0); // separator
    }
    encoded
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8081")?;
    println!("HTTP/2 Toy Server running on 127.0.0.1:8081");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            thread::spawn(move || handle_connection(&mut stream));
        }
    }

    Ok(())
}

fn handle_connection(stream: &mut impl Read + Write) {
    let mut buf = BytesMut::with_capacity(4096);
    let mut stream_map = HashMap::new();

    loop {
        let mut tmp = [0u8; 1024];
        let bytes_read = match stream.read(&mut tmp) {
            Ok(0) => break, // client closed
            Ok(n) => n,
            Err(_) => break,
        };
        buf.extend_from_slice(&tmp[..bytes_read]);

        while buf.len() >= FRAME_HEADER_LEN {
            let mut header = buf.split_to(FRAME_HEADER_LEN);
            let len = ((header.get_u8() as usize) << 16)
                    | ((header.get_u8() as usize) << 8)
                    | (header.get_u8() as usize);
            let frame_type = header.get_u8();
            let _flags = header.get_u8();
            let stream_id = header.get_u32() & 0x7FFFFFFF;

            if buf.len() < len {
                // Incomplete frame body
                buf.reserve(len);
                buf.unsplit(header);
                break;
            }

            let payload = buf.split_to(len);

            match frame_type {
                HEADERS => {
                    let headers = decode_headers(&payload);
                    println!("[Stream {}] Received HEADERS: {:?}", stream_id, headers);
                    stream_map.insert(stream_id, headers);

                    // Respond with HEADERS + DATA frame
                    let mut response_headers = HashMap::new();
                    response_headers.insert(":status", "200");
                    response_headers.insert("content-type", "text/plain");

                    let headers_bytes = encode_headers(&response_headers);
                    send_frame(stream, HEADERS, 0x4, stream_id, &headers_bytes); // END_HEADERS

                    let body = b"Hello from toy HTTP/2 server!";
                    send_frame(stream, DATA, 0x1, stream_id, body); // END_STREAM
                }
                DATA => {
                    println!("[Stream {}] Received DATA ({} bytes)", stream_id, payload.len());
                }
                _ => {
                    println!("[Stream {}] Unknown frame type: {}", stream_id, frame_type);
                }
            }
        }
    }
}

fn send_frame(stream: &mut impl Write, frame_type: u8, flags: u8, stream_id: u32, payload: &[u8]) {
    let len = payload.len();
    let mut header = Vec::with_capacity(FRAME_HEADER_LEN);

    header.push(((len >> 16) & 0xFF) as u8);
    header.push(((len >> 8) & 0xFF) as u8);
    header.push((len & 0xFF) as u8);
    header.push(frame_type);
    header.push(flags);
    header.extend_from_slice(&(stream_id & 0x7FFFFFFF).to_be_bytes());

    stream.write_all(&header).unwrap();
    stream.write_all(payload).unwrap();
    stream.flush().unwrap();
}

