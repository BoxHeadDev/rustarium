use bson::Document;
use ron;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::Write;
use std::io::{BufReader, BufWriter, Cursor};
use std::str;

// Dervie debug (:?)
// Move type (direction)
#[derive(Debug, Serialize, Deserialize)]
struct Move {
    direction: String,
    distance: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1000 Move values

    // Create file
    let file_create = fs::File::create("move.bson").unwrap();
    let mut buf_writer = BufWriter::new(file_create);

    let mut seralized_count = 0;
    // Loop 1000
    for i in 0..1000 {
        // Move value
        // Ser BSON
        let value = Move {
            direction: "forward".to_string(),
            distance: i,
        };

        // Add to File
        let doc = bson::to_document(&value).unwrap();

        let _ = doc.to_writer(&mut buf_writer);

        seralized_count += 1;
    }

    println!("serialized: {}", seralized_count);

    buf_writer.flush()?;

    // Read file
    let file_open = fs::File::open("move.bson").unwrap();
    let mut buf_reader = BufReader::new(file_open);

    // len ??
    let mut deserialized_count = 0;

    // de BSON
    while let Ok(deserialized) = Document::from_reader(&mut buf_reader) {
        let _mv: Move = bson::from_document(deserialized)?;
        deserialized_count += 1;
    }

    println!("deserialized: {}", deserialized_count);

    Ok(())
}
