// Import standard library modules for environment variables, error handling, and file reading
use std::env;
use std::error::Error;
use std::fs;

// Define the main logic function, which takes a Config and returns a Result (error handling)
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // Read the file contents into a string; use `?` to propagate any error
    let contents = fs::read_to_string(config.file_path)?;

    // Choose between case-sensitive or case-insensitive search based on the config
    let result = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    // Print each line that matched the query
    for line in result {
        println!("{line}");
    }

    // Return Ok to indicate successful execution
    Ok(())
}

// Struct to hold configuration data: the search query, file path, and case-sensitivity flag
pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}

impl Config {
    // Build a Config object from command-line arguments
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        // Ensure at least 3 arguments are provided (program name, query, file path)
        if args.len() < 3 {
            return Err("Not enough argmuments");
        }

        // Clone the query and file path from the arguments
        let query = args[1].clone();
        let file_path = args[2].clone();

        // Determine whether to ignore case by checking the environment variable
        let ignore_case = env::var("IGNORE_CASE").is_ok();

        // Return a new Config instance
        Ok(Config {
            query,
            file_path,
            ignore_case,
        })
    }
}

// Case-sensitive search: find lines that contain the query
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut result = Vec::new();

    // Iterate through each line and collect matches
    for line in contents.lines() {
        if line.contains(query) {
            result.push(line);
        }
    }

    result
}

// Case-insensitive search: find lines that contain the query, ignoring case
pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    // Convert query to lowercase
    let query = query.to_lowercase();

    let mut result = Vec::new();

    // Compare each line in lowercase
    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            result.push(line);
        }
    }

    result
}

// Unit tests to verify the search functions
#[cfg(test)]
mod tests {
    use super::*;

    // Test case-sensitive search for one match
    #[test]
    fn one_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
                        ";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    // Test case-insensitive search for multiple matches
    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }
}
