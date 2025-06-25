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
        direction: String::from("Backwards"),
        distance: 34,
    };

    let buffer: Vec<u8> = serde_json::to_vec(&a)?;

    let json_str = str::from_utf8(&buffer)?;

    let b: Move = serde_json::from_str(&json_str)?;

    println!("{:?} to {:?}", a, b);

    Ok(())
}
