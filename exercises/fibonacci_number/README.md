# Fibonacci Number Generator Challenge

Welcome to this Rust programming challenge! In this task, you'll build a Fibonacci number generator that calculates the n-th Fibonacci number based on user input. This project will help you practice recursion, iteration, pattern matching, and Rust’s type system.

## Challenge Overview

Your goal is to write a Rust program that:

- Prompts the user to enter a positive integer (n).
- Reads and validates the user input.
- Computes the n-th Fibonacci number using both iterative and recursive approaches.
- Prints the result.

## Understanding Fibonacci Numbers

The Fibonacci sequence is defined as:

## Getting Started

1. Implement the Logic
- Use user input (io::stdin) to get the value of n.
- Ensure n is a valid positive integer.
- Implement two ways to calculate Fibonacci:
  - Iterative (efficient)
  - Recursive (demonstrates recursion, but inefficient for large n)

2. Handling Edge Cases
Your program should handle:
- Invalid input (e.g., letters, negative numbers).
- Large numbers (consider using u128 for big Fibonacci numbers).
- Performance differences between recursion and iteration.

## Bonus Challenges
If you complete the basic challenge, try these additional tasks:

- Memoization (Caching): Optimize the recursive solution to store previously computed Fibonacci numbers.
- Compare Execution Time: Measure and display the execution time for both approaches.
- Compute the Entire Sequence: Print the full Fibonacci sequence up to n.

