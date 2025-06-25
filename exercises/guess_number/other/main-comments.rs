// Import the `Rng` trait to enable random number generation.
// Import `Ordering` enum to compare values.
// Import the `io` module to handle user input.

fn main() {
    // Display a welcome message to the user. "Guess the number!"

    // Start an infinite loop to allow multiple guesses.

    // Generate a random number between 0 and 100 (inclusive). secret_number

    // (For testing/debugging purposes) Print the secret number. "The secret number is: ?"

    // Prompt the user to input their guess. "Please input your guess."

    // Create a new, empty mutable String to store the user's input. guess

    // Read the user's input from standard input and store it in `guess`.
    // If reading fails, the program will panic with an error message. "Failed to read line"

    // Attempt to parse the input string into a 32-bit unsigned integer.
    // If parsing fails (e.g., input is not a number), skip the rest of the loop. guess

    // Echo the guessed number back to the user. "You guesses: ?"

    // Compare the guessed number to the secret number.
    // If guess is less than the secret number, inform the user. "Too Small!"
    // If guess is greater than the secret number, inform the user. "Too Big!"
    // If guess equals the secret number, congratulate and exit loop. "You Win!"
}
