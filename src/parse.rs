use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;
use rand::Rng;
use rust_decimal::prelude::*;

// TODO: figure out how to elegantly handle nested dice rolls
// (e.g. 2d3d4. Rendering as (2, 1)d4 loses something; so does rendering as (4, 2, 4). Solve by having a default view and then offering an option to display all individual roll results, probably.)

enum Binop {
    Dice,
    Plus,
    Minus,
    Times,
    Divide,
    Mod,
}

enum Unop {
    Plus,
    Minus,
}

#[derive(Parser)]
#[grammar = "dice.pest"]
struct DiceParser;

fn roll_die(sides: i128) -> Decimal {
    let roll = rand::thread_rng().gen_range(1..=sides);
    Decimal::from(roll)
}

fn roll_dice(number: Decimal, sides: Decimal) -> (Decimal, Vec<Decimal>) {
    assert_eq!(number, number.floor(), "Attempted to roll non-integer number of dice.");
    assert_eq!(sides, sides.floor(), "Attempted to roll dice with non-integer number of sides.");
    assert!(!number.is_sign_negative(), "Attempted to roll negative number of dice.");
    assert!(sides.is_sign_positive(), "Attempted to roll dice with non-positive number of sides.");
    if number.is_zero() {
        return (Decimal::from(0), Vec::new())
    }

    let number_as_int = number.abs().mantissa();
    let sides_as_int = sides.abs().mantissa();

    let mut sum = Decimal::from(0);
    let mut rolls = Vec::new();
    for _ in 0..number_as_int {
        let roll = roll_die(sides_as_int);
        sum += roll;
        rolls.push(roll);
    }

    (sum, rolls)
}

fn parse_number(number: Pair<Rule>) -> (Decimal, String) {
    assert_eq!(number.as_rule(), Rule::number, "Called parse_number on non-number.");

    let mut number_as_string = String::from(number.as_str());
    number_as_string.retain(|c| !c.is_whitespace());
    
    (Decimal::from_str(&number_as_string).unwrap(), number_as_string)
}

fn parse_binop(binop: Pair<Rule>) -> Binop {
    assert_eq!(binop.as_rule(), Rule::binop, "Called parse_binop on non-binop.");

    let internal_binop = binop.into_inner().next().unwrap();
    
    match internal_binop.as_rule() {
        Rule::dice => Binop::Dice,
        Rule::plus_binop => Binop::Plus,
        Rule::minus_binop => Binop::Minus,
        Rule::times => Binop::Times,
        Rule::divide => Binop::Divide,
        Rule::modulus => Binop::Mod,
        _ => panic!("Non-binop found inside binop token."),
    }
}

fn parse_unop(unop: Pair<Rule>) -> Unop {
    assert_eq!(unop.as_rule(), Rule::unop, "Called parse_unop on non-unop.");

    let internal_unop = unop.into_inner().next().unwrap();

    match internal_unop.as_rule() {
        Rule::plus_unop => Unop::Plus,
        Rule::minus_unop => Unop::Minus,
        _ => panic!("Non-unop found inside unop token."),
    }
}

fn parse_paren_block(paren_block: Pair<Rule>) -> (Decimal, String, Vec<String>, Vec<Vec<Decimal>>) {
    assert_eq!(paren_block.as_rule(), Rule::paren_block, "Called parse_paren_block on non-paren-block.");

    let mut inside = paren_block.into_inner();
    let (mut result, mut processed_string, mut original_rolls, mut roll_vals) = parse_legitimate_sequence(inside.next().unwrap());

    let mut next = inside.next();
    while next != None {
        let binop = parse_binop(next.unwrap());
        let (next_result, next_str_segment, mut next_rolls, mut next_roll_vals) = parse_legitimate_sequence(inside.next().unwrap());

        match binop {
            Binop::Dice => {
                let (new_result, rolls) = roll_dice(result, next_result);
                result = new_result;
                processed_string = format!("{}d{}", processed_string, next_str_segment);
                original_rolls.push(processed_string.clone());
                roll_vals.push(rolls);
            }
            Binop::Plus => {
                result += next_result;
                processed_string = format!("{}+{}", processed_string, next_str_segment);
            }
            Binop::Minus => {
                result -= next_result;
                processed_string = format!("{}-{}", processed_string, next_str_segment);
            }
            Binop::Times => {
                result *= next_result;
                processed_string = format!("{}*{}", processed_string, next_str_segment);
            }
            Binop::Divide => {
                result /= next_result;
                processed_string = format!("{}/{}", processed_string, next_str_segment);
            }
            Binop::Mod => {
                result %= next_result;
                processed_string = format!("{}%{}", processed_string, next_str_segment);
            }
        }

        original_rolls.append(&mut next_rolls);
        roll_vals.append(&mut next_roll_vals);

        next = inside.next();
    }

    (result, format!("({})", processed_string), original_rolls, roll_vals)
}

fn parse_non_operator(non_operator: Pair<Rule>) -> (Decimal, String, Vec<String>, Vec<Vec<Decimal>>) {
    assert_eq!(non_operator.as_rule(), Rule::non_operator, "Called parse_non_operator on non-paren-block.");

    let inside = non_operator.into_inner().next().unwrap();

    match inside.as_rule() {
        Rule::number => {
            let (number, string) = parse_number(inside);
            (number, string, Vec::new(), Vec::new())
        }
        Rule::paren_block => parse_paren_block(inside),
        _ => panic!("Non-operator token inside isn't a number or paren block.")
    }
}

fn parse_paired_unop(paired_unop: Pair<Rule>) -> (Decimal, String, Vec<String>, Vec<Vec<Decimal>>) {
    assert_eq!(paired_unop.as_rule(), Rule::paired_unop, "Called parse_paired_unop on non-paired-unop.");

    let mut inside = paired_unop.into_inner();

    let unop = inside.next().unwrap();
    let (result, string, original_rolls, roll_vals) = parse_non_operator(inside.next().unwrap());

    match parse_unop(unop) {
        Unop::Plus => (result, format!("+{}", string), original_rolls, roll_vals),
        Unop::Minus => (result * Decimal::from(-1), format!("-{}", string), original_rolls, roll_vals),
    }
}

fn parse_non_binop(non_binop: Pair<Rule>) -> (Decimal, String, Vec<String>, Vec<Vec<Decimal>>) {
    assert_eq!(non_binop.as_rule(), Rule::non_binop, "Called parse_non_binop on non-non-binop.");

    let inside = non_binop.into_inner().next().unwrap();
    
    match inside.as_rule() {
        Rule::number => {
            let (number, string) = parse_number(inside);
            (number, string, Vec::new(), Vec::new())
        }
        Rule::paren_block => parse_paren_block(inside),
        Rule::paired_unop => parse_paired_unop(inside),
        _ => panic!("Non-binop token inside isn't a number, paren block, or paired unop.")
    }
}

fn parse_legitimate_sequence(sequence: Pair<Rule>) -> (Decimal, String, Vec<String>, Vec<Vec<Decimal>>) {
    assert_eq!(sequence.as_rule(), Rule::legitimate_sequence, "Called parse_legitimate_sequence on non-legitimate-sequence.");

    let mut inside = sequence.into_inner();
    let (mut result, mut processed_string, mut original_rolls, mut roll_vals) = parse_non_binop(inside.next().unwrap());

    let mut next = inside.next();
    while next != None {
        let binop = parse_binop(next.unwrap());
        let (next_result, next_str_segment, mut next_rolls, mut next_roll_vals) = parse_non_binop(inside.next().unwrap());

        match binop {
            Binop::Dice => {
                let (new_result, rolls) = roll_dice(result, next_result);
                result = new_result;
                original_rolls.push(format!("{}d{}", processed_string, next_str_segment));
                processed_string = String::from("{}");
                roll_vals.push(rolls);
            }
            Binop::Plus => {
                result += next_result;
                processed_string = format!("{}+{}", processed_string, next_str_segment);
            }
            Binop::Minus => {
                result -= next_result;
                processed_string = format!("{}-{}", processed_string, next_str_segment);
            }
            Binop::Times => {
                result *= next_result;
                processed_string = format!("{}*{}", processed_string, next_str_segment);
            }
            Binop::Divide => {
                result /= next_result;
                processed_string = format!("{}/{}", processed_string, next_str_segment);
            }
            Binop::Mod => {
                result %= next_result;
                processed_string = format!("{}%{}", processed_string, next_str_segment);
            }
        }

        original_rolls.append(&mut next_rolls);
        roll_vals.append(&mut next_roll_vals);

        next = inside.next();
    }

    (result, processed_string, original_rolls, roll_vals)
}

// Output format: result (as decimal), processed string (with placeholders where rolls were), vec of original roll texts, vec of rolls
fn parse_full_expression(mut tree: Pairs<Rule>) -> (Decimal, String, Vec<String>, Vec<Vec<Decimal>>) {
    let full_expression = tree.next().unwrap();
    let sequence = full_expression.into_inner().next().unwrap();
    
    parse_legitimate_sequence(sequence)
}

fn clean_input(input: &str) -> String {
    let mut clean = String::from(input);
    clean.retain(|c| "0123456789d+-*/%() ".contains(c));

    clean
}

pub fn parse_input(input: &str) -> (Decimal, String, Vec<String>, Vec<Vec<Decimal>>) {
    let cleaned = clean_input(input);
    let full_expression = DiceParser::parse(Rule::full_expression, &cleaned)
        .expect("Ill-formed input");
    parse_full_expression(full_expression)
}
