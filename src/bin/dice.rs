use argh::FromArgs;
use dice::parse::parse_input;
use rust_decimal::prelude::*;

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

    if args.roll.is_empty() {
        println!("Attempted to invoke dice roller CLI with no roll argument. In the future this may (or may not) launch the GUI, but right now you're just getting this error message.");
    } else {
        match parse_input(&args.roll.join(" ")) {
            Ok(results) => {
                if args.verbose {
                    println!("{:?}", results);
                    println!("{}", format_string_with_rolls(&results.processed_string, results.original_roll_texts));
                    println!("{}", format_string_with_results(&results.processed_string, results.rolls));
                } else {
                    println!("{}", results.value)
                }
            }
            Err(e) => println!("{}", e),
        }
    }
}
