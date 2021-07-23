use argh::FromArgs;
use fluorite::parse::parse_input;
use fluorite::{format_string_with_results, format_string_with_rolls};
use std::io::{Read, stdin};

/// Roll dice via string input.
#[derive(FromArgs)]
struct Args {
    /// display full roll output
    #[argh(switch, short = 'v')]
    verbose: bool,
    #[argh(positional)]
    roll: Vec<String>,
}

fn main() {
    let args: Args = argh::from_env();

    let input = if !args.roll.is_empty() {
        args.roll.join(" ")
    } else {
        let mut buffer = String::new();
        match stdin().read_to_string(&mut buffer) {
            Ok(_) => (),
            Err(_) => println!("Failed to read from stdin."),
        };
        buffer
    };

    for line in input.split('\n').filter(|line| line != &"") {
        match parse_input(&line) {
            Ok(results) => {
                if args.verbose {
                    println!("Original roll text: {}", format_string_with_rolls(&results.processed_string, results.original_roll_texts));
                    println!("With results: {}", format_string_with_results(&results.processed_string, results.rolls));
                    println!("Total: {}", results.value);
                } else {
                    println!("{}", results.value);
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
}
