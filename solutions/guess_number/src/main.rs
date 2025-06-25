// Import the `Rng` trait to enable random number generation.
use rand::Rng;

// Import `Ordering` enum to compare values.
use std::cmp::Ordering;

// Import the `io` module to handle user input.
use std::io;

fn main() {
    // Display a welcome message to the user. "Guess the number!"
    println!("Guess the number!");

    // Start an infinite loop to allow multiple guesses.
    loop {
        // Generate a random number between 0 and 100 (inclusive). secret_number
        let secret_number = rand::thread_rng().gen_range(0..=100);

        // (For testing/debugging purposes) Print the secret number. "The secret number is: ?"
        println!("The secret number is: {secret_number}");

        // Prompt the user to input their guess. "Please input your guess."
        println!("Please input your guess.");

        // Create a new, empty mutable String to store the user's input. guess
        let mut guess = String::new();

        // Read the user's input from standard input and store it in `guess`.
        // If reading fails, the program will panic with an error message. "Failed to read line"
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        // Attempt to parse the input string into a 32-bit unsigned integer.
        // If parsing fails (e.g., input is not a number), skip the rest of the loop. guess
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        // Echo the guessed number back to the user. "You guesses: ?"
        println!("You guessed: {}", guess);

        // Compare the guessed number to the secret number.
        match guess.cmp(&secret_number) {
            // If guess is less than the secret number, inform the user. "Too Small!"
            Ordering::Less => println!("Too Small!"),
            // If guess is greater than the secret number, inform the user. "Too Big!"
            Ordering::Greater => println!("Too Big!"),
            // If guess equals the secret number, congratulate and exit loop. "You Win!"
            Ordering::Equal => {
                println!("You Win!");
                break;
            }
        }
    }
}
