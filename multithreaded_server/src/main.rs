use hello::ThreadPool; // Import the custom thread pool implementation from the `hello` crate/module
use std::{
    fs,
    io::{prelude::*, BufReader}, // Used for reading lines from the stream
    net::{TcpListener, TcpStream}, // For network connections
    thread,
    time::Duration,
};

fn main() {
    // Bind the TCP listener to address 127.0.0.1:7878
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // Create a thread pool with 4 worker threads
    let pool = ThreadPool::new(4);

    // Accept and handle only 2 incoming connections before shutting down
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        // Submit the connection handling task to the thread pool
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

// Handle the client connection by reading the request and sending a response
fn handle_connection(mut stream: TcpStream) {
    // Wrap the stream in a buffered reader to enable convenient line-by-line reading
    let buf_reader = BufReader::new(&stream);
    // Read the first request line
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // Match the request line and determine the appropriate response
    let (status_line, filename) = match &request_line[..] {
        // Serve the homepage for root path
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),

        // Simulate a delayed response to test server concurrency handling
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }

        // Respond with 404 for all other (unrecognized) paths
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    // Read the content of the corresponding HTML file
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    // Format the full HTTP response
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // Send the response to the client
    stream.write_all(response.as_bytes()).unwrap();
}
