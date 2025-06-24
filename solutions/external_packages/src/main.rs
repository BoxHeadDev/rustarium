// TO DO: Add the Rand package
use rand::Rng;

// TO DO: Change to a nested path
// use std::{cmp::Ordering, io};

// TO DO: Change to a nested path
// use std::io::{self, Write};

// TO DO: bring all public items defined in a path into scope
// use std::collections::*;

fn main() {
    let secret_number = rand::thread_rng().gen_range(1..=100);
}
