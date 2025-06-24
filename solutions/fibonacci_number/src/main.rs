use std::collections::HashMap;
use std::io;
use std::time::Instant;

/// Iterative approach - Efficient O(n) complexity
fn fibonacci_iterative(n: u32) -> u128 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }

    let mut a = 0;
    let mut b = 1;

    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }

    b
}

/// Recursive approach - Inefficient O(2^n) complexity
fn fibonacci_recursive(n: u32) -> u128 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci_recursive(n - 1) + fibonacci_recursive(n - 2),
    }
}

/// Optimized Recursive approach with memoization - O(n) complexity
fn fibonacci_memoized(n: u32, memo: &mut HashMap<u32, u128>) -> u128 {
    if let Some(&result) = memo.get(&n) {
        return result;
    }

    let result = match n {
        0 => 0,
        1 => 1,
        _ => fibonacci_memoized(n - 1, memo) + fibonacci_memoized(n - 2, memo),
    };

    memo.insert(n, result);
    result
}

fn main() {
    println!("🌀 Fibonacci Number Generator");

    loop {
        println!("Enter a positive integer (n) to compute the Fibonacci number (or type 'exit' to quit):");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let input = input.trim();

        // Allow user to exit
        if input.eq_ignore_ascii_case("exit") {
            println!("Goodbye! 👋");
            break;
        }

        // Validate user input
        match input.parse::<u32>() {
            Ok(n) => {
                let start_time = Instant::now();
                let result_iterative = fibonacci_iterative(n);
                let duration_iterative = start_time.elapsed();

                let start_time = Instant::now();
                let result_recursive = fibonacci_recursive(n);
                let duration_recursive = start_time.elapsed();

                let mut memo = HashMap::new();
                let start_time = Instant::now();
                let result_memoized = fibonacci_memoized(n, &mut memo);
                let duration_memoized = start_time.elapsed();

                println!(
                    "🔄 Using iteration: Fibonacci({n}) = {result_iterative} (Time: {:?})",
                    duration_iterative
                );
                println!(
                    "🧵 Using recursion: Fibonacci({n}) = {result_recursive} (Time: {:?})",
                    duration_recursive
                );
                println!(
                    "⚡ Using memoized recursion: Fibonacci({n}) = {result_memoized} (Time: {:?})",
                    duration_memoized
                );
            }
            Err(_) => println!("❌ Invalid input! Please enter a valid positive integer."),
        }
    }
}
