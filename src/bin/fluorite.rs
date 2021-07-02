use argh::FromArgs;
use fluorite::parse::parse_input;
use rust_decimal::prelude::*;
use std::io::stdin;

/// Roll dice via string input.
#[derive(FromArgs)]
struct Args {
    /// display full roll output
    #[argh(switch, short = 'v')]
    verbose: bool,
    #[argh(positional)]
    roll: Vec<String>,
}

// This function is almost certainly horribly inefficient, with all the strings it allocates. Improvements wanted.
fn format_string_with_rolls(string: &str, rolls: Vec<String>) -> String {
    let mut formatted = String::from(string);
    for roll in rolls {
        formatted = formatted.replacen("{}", &format!("{}", roll), 1);
    }

    formatted
}

// This function is almost certainly horribly inefficient, with all the strings it allocates. Improvements wanted.
fn format_string_with_results(string: &str, result_vecs: Vec<Vec<Decimal>>) -> String {
    let mut formatted = String::from(string);
    for roll_results in result_vecs {
        let joined = roll_results.iter().map(|dec| dec.to_string()).collect::<Vec<String>>().join(", ");
        formatted = formatted.replacen("{}", &format!("[{}]", joined), 1);
    }

    formatted
}

fn main() {
    let args: Args = argh::from_env();

    let input = if !args.roll.is_empty() {
        args.roll.join(" ")
    } else {
        let mut buffer = String::new();
        match stdin().read_line(&mut buffer) {
            Ok(_) => (),
            Err(_) => println!("Failed to read from stdin."),
        };
        buffer
    };

    match parse_input(&input) {
        Ok(results) => {
            if args.verbose {
                println!("Original roll text: {}", format_string_with_rolls(&results.processed_string, results.original_roll_texts));
                println!("With results: {}", format_string_with_results(&results.processed_string, results.rolls));
                println!("Total: {}", results.value);
            } else {
                println!("{}", results.value);
            }
        }
        Err(e) => println!("{}", e),
    }
}
