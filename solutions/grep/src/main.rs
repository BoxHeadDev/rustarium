// Import the standard library's environment module to access command-line arguments
use std::env;
// Import the process module for exiting the program with a status code
use std::process;

// Import the `Config` struct and possibly other items from the `minigrep` crate or module
use minigrep::Config;

// The main function, the entry point of the program
fn main() {
    // Collect command-line arguments into a vector of strings
    let args: Vec<String> = env::args().collect();

    // Attempt to create a Config instance from the arguments
    // If it fails, print an error message and exit with status code 1
    let config = Config::build(&args).unwrap_or_else(|err| {
        // Print the parsing error to standard error
        eprintln!("{err}");
        // Exit the program with a non-zero status code
        process::exit(1);
    });

    // Print the search query to standard output
    println!("Search Query: {}", config.query);
    // Print the file path to standard output
    println!("File Path: {}", config.file_path);

    // Attempt to run the minigrep functionality with the given config
    // If an error occurs, print it and exit with status code 1
    if let Err(e) = minigrep::run(config) {
        // Print the runtime error to standard error
        eprintln!("{e}");
        // Exit the program with a non-zero status code
        process::exit(1);
    }
}
