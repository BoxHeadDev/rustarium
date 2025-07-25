use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;

pub mod client;
pub use crate::handle_connection;
pub use crate::send_frame;
pub use client::*;

/// Frame types
const HEADERS: u8 = 0x1;
const DATA: u8 = 0x0;

const FRAME_HEADER_LEN: usize = 9;

/// Encode headers in a toy HPACK-like format
fn encode_headers(headers: &HashMap<&str, &str>) -> Vec<u8> {
    let mut encoded = Vec::new();
    for (k, v) in headers {
        encoded.extend_from_slice(k.as_bytes());
        encoded.push(b':');
        encoded.extend_from_slice(v.as_bytes());
        encoded.push(0); // separator
    }
    encoded
}

/// Parse toy header payload
fn decode_headers(payload: &[u8]) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    let mut start = 0;
    for i in 0..payload.len() {
        if payload[i] == 0 {
            if let Some(colon_pos) = payload[start..i].iter().position(|&b| b == b':') {
                let key = String::from_utf8_lossy(&payload[start..start + colon_pos]);
                let value = String::from_utf8_lossy(&payload[start + colon_pos + 1..i]);
                headers.insert(key.to_string(), value.to_string());
            }
            start = i + 1;
        }
    }
    headers
}

pub struct Http2Response {
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub fn send_http2_request(
    host: &str,
    port: u16,
    path: &str,
) -> Result<Http2Response, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect((host, port))?;

    let mut headers = HashMap::new();
    headers.insert(":method", "GET");
    headers.insert(":path", path);
    headers.insert(":scheme", "http");
    headers.insert("user-agent", "toy-client/0.1");

    let payload = encode_headers(&headers);
    send_frame(&mut stream, HEADERS, 0x4, 1, &payload); // END_HEADERS

    let mut buf = [0u8; 1024];
    let mut total_read = 0;
    let mut response_headers = HashMap::new();
    let mut response_body = String::new();

    while let Ok(n) = stream.read(&mut buf[total_read..]) {
        if n == 0 {
            break;
        }
        total_read += n;

        let mut cursor = 0;
        while total_read - cursor >= FRAME_HEADER_LEN {
            let len = ((buf[cursor] as usize) << 16)
                | ((buf[cursor + 1] as usize) << 8)
                | (buf[cursor + 2] as usize);
            let frame_type = buf[cursor + 3];
            let _flags = buf[cursor + 4];
            let stream_id = u32::from_be_bytes([
                buf[cursor + 5],
                buf[cursor + 6],
                buf[cursor + 7],
                buf[cursor + 8],
            ]) & 0x7FFFFFFF;

            if total_read - cursor - FRAME_HEADER_LEN < len {
                break;
            }

            let payload = &buf[cursor + FRAME_HEADER_LEN..cursor + FRAME_HEADER_LEN + len];

            match frame_type {
                HEADERS => {
                    response_headers = decode_headers(payload);
                }
                DATA => {
                    response_body.push_str(&String::from_utf8_lossy(payload));
                }
                _ => {}
            }

            cursor += FRAME_HEADER_LEN + len;
        }

        if total_read >= 1024 {
            break;
        }
    }

    Ok(Http2Response {
        headers: response_headers,
        body: response_body,
    })
}

/// Send an HTTP/2 frame
fn send_frame(stream: &mut TcpStream, frame_type: u8, flags: u8, stream_id: u32, payload: &[u8]) {
    let len = payload.len();
    let mut header = Vec::with_capacity(9);

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

fn main() {
    client::send_http2_request("127.0.0.1", 8081, "/");
}
