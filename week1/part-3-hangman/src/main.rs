// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    // println!("random word: {}", secret_word);

    // Your code here! :)
    let n = secret_word.len();
    println!("{}", "Welcome to CS110L Hangman!");
    // I don't know how to do a better init.
    let mut cur_word = Vec::new();
    cur_word.resize(n, '-');
    let mut guesses_his = String::new();
    let mut chances = NUM_INCORRECT_GUESSES;
    
    while chances > 0 {
        let cur_word_p : String = cur_word.iter().collect();
        println!("The word so far is {}", cur_word_p);
        println!("You have guessed the following letters: {}", guesses_his);
        println!("You have {} guesses left", chances);
        print!("Please guess a letter:");
        io::stdout()
            .flush()
            .expect("Error flushing stdout.");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Error reading line."); 
        let mut ok = false;
        let c = guess.chars().nth(0).unwrap();
        for i in 0..n {
            if secret_word_chars[i] == c {
                ok = true;
                cur_word[i] = secret_word_chars[i];
            }
        }
        guesses_his.push(c);
        if !ok {
            println!("Sorry, that letter is not in the word");
            chances -= 1;
        } else {
            let mut succ = true;
            for i in 0..n {
                if cur_word[i] == '-' {
                    succ = false;
                    break;
                }
            }
            if succ {
                println!("Congratulations you guessed the secret word: {}!", secret_word);
                return;
            }
        }
        println!();
    }
    println!("Sorry, you ran out of guesses!");
}
