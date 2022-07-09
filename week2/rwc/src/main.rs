use std::env;
use std::process;
use std::fs::File; // For read_file_lines()
use std::io::{self, BufRead}; // For read_file_lines()

/// Reads the file at the supplied path, and returns a vector of strings.
fn read_file_lines(filename: &String) -> Result<Vec<Vec<char>>, io::Error> {
    let file = File::open(filename)?;
    let mut res : Vec<Vec<char>> = Vec::new();
    for line in io::BufReader::new(file).lines() {
        let line_str = line?;
        // do something with line_str
        let line_chars : Vec<char> = line_str.chars().collect();
        res.push(line_chars);
    }
    Ok(res)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    // Your code here :)

    let lines_result = read_file_lines(&filename);
    assert!(lines_result.is_ok());
    let lines = lines_result.unwrap();
    let lines_n = lines.len();
    let mut words_n = 0;
    let mut chars_n = 0;

    for i in 0..lines_n-1 {
        let s_len = lines[i].len();
        let mut lch : bool = false;
        for j in 0..s_len-1 {
            chars_n += 1;
            if lines[i][j].is_alphabetic() || lines[i][j] == '\'' {
                if lch == false {
                    words_n += 1;
                }
                lch = true;
            } else {
                lch = false;
            }
        }
        chars_n += 1;
    }

    println!("  {}  {}  {}  {}", lines_n, words_n, chars_n, filename);
}
