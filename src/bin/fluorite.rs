use argh::FromArgs;
use fluorite::parse::{clean_input, parse_input};
use std::io::{stdin, Read};

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
                    println!("Input: {}", clean_input(line));
                    println!("Rolled: {}", results.processed_string);
                    println!("Result: {}", results.value);
                } else {
                    println!("{}", results.value);
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
}
