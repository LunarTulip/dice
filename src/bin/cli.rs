use argh::FromArgs;
use dice::parse;
use rust_decimal::prelude::*;

/// Roll dice via string input.
#[derive(FromArgs)]
struct Args {
    #[argh(positional)]
    roll: String,
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

    let results = parse::parse_input(&args.roll);
    println!("{:?}", results);
    println!("{}", format_string_with_rolls(&results.1, results.2));
    println!("{}", format_string_with_results(&results.1, results.3));
}
