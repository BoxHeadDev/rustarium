// TO DO: Add the Rand package
// ??

// TO DO: Change to a nested path
use std::cmp::Ordering;
use std::io;

// TO DO: Change to a nested path
use std::io;
use std::io::Write;

// TO DO: bring all public items defined in a path into scope
// use std::collections

fn main() {
    let secret_number = rand::thread_rng().gen_range(1..=100);
}
