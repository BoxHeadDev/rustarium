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
        direction: String::from("Left"),
        distance: 23,
    };

    let ron_str = ron::to_string(&a)?;

    let buffer: Vec<u8> = ron_str.clone().into_bytes();

    let json_str = str::from_utf8(&buffer)?;

    let b: Move = ron::de::from_str(json_str)?;

    println!("{:?} to {:?}", a, b);

    Ok(())
}
