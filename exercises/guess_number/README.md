# Guessing Game Challenge
## Overview

Welcome to the Guessing Game Challenge! This exercise is designed to introduce you to Rust by building a small interactive game. Through this project, you'll practice fundamental Rust concepts such as variables, user input, loops, error handling, and external crates.

## Challenge Description

You will create a number guessing game where:
- The program randomly selects a number between 1 and 100.
- The user is prompted to guess the number.
- The program provides feedback on whether the guess is too high, too low, or correct.
- If the guess is incorrect, the user is prompted to guess again.
- The game continues until the user guesses the correct number.
- Upon guessing correctly, the game prints a congratulatory message and exits.

## Learning Objectives

By completing this challenge, you will:
- Learn how to set up a new Rust project using Cargo.
- Work with variables and mutability in Rust.
- Use standard input/output (I/O) to read user input.
- Learn about random number generation using an external crate.
- Implement conditional logic with match expressions.
- Utilize loops to keep the game running until the correct guess is made.
- Handle potential input errors and prevent program crashes.

## Getting Started

1. Create a New Rust Project
- Use Cargo to create a new Rust project
2. Add Dependencies
- Modify the Cargo.toml file to include the rand crate, which will be used to generate a random number.
3. Implement the Game
- Prompt the user for input.
- Read and process the input.
- Compare the guess with the randomly generated number.
- Provide feedback to the user.
- Repeat until the user guesses correctly.
4. Handle Edge Cases
- Ensure the user can only enter valid numbers.
- Prevent crashes when non-numeric input is provided.
- Make sure the user can continue guessing after an invalid input.

## Hints
- Use rand::thread_rng().gen_range(1..=100) to generate a random number.
- Use io::stdin().read_line(&mut guess) to read user input.
- Convert the input string to a number using .trim().parse::<u32>().
- Use a loop to keep the game running until the correct guess is made.
- Utilize the Ordering enum (Less, Greater, Equal) for comparing values.
- Use match to handle different outcomes.

## Extra Challenges

Want to take it a step further? Try implementing these additional features:
- Track the number of guesses the user makes.
- Give hints by stating how far off the guess is.
- Allow the user to restart the game without quitting.
- Implement difficulty levels (e.g., easy: 1-50, hard: 1-500).

