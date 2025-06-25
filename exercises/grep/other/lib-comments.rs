// Import standard library modules for environment variables, error handling, and file reading

// Define the main logic function, which takes a Config and returns a Result (error handling)

// Read the file contents into a string; use `?` to propagate any error

// Choose between case-sensitive or case-insensitive search based on the config

// Print each line that matched the query

// Return Ok to indicate successful execution

// Struct to hold configuration data: the search query, file path, and case-sensitivity flag

// Build a Config object from command-line arguments

// Ensure at least 3 arguments are provided (program name, query, file path)

// Clone the query and file path from the arguments

// Determine whether to ignore case by checking the environment variable

// Return a new Config instance

// Case-sensitive search: find lines that contain the query

// Variable to store result

// Iterate through each line and collect matches

// Case-insensitive search: find lines that contain the query, ignoring case

// Convert query to lowercase

// Variable to store result

// Compare each line in lowercase

// Unit tests to verify the search functions

// Test case-sensitive search for one match

// Test case-insensitive search for multiple matches
