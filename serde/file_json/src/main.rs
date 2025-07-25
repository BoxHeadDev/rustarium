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
    let a = Move {
        direction: String::from("Forward"),
        distance: 2,
    };

    // a = serialize Move to file
    let file_create = fs::File::create("move.json")?;
    let buf_writer = BufWriter::new(file_create);
    serde_json::to_writer(buf_writer, &a)?;

    // b = deserialize Move from file
    let file_open = fs::File::open("move.json")?;
    let buf_reader = BufReader::new(file_open);
    let b: Move = serde_json::from_reader(buf_reader)?;

    // print a and b
    println!("{:?} to {:?}", a, b);

    Ok(())
}
