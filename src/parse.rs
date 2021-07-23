use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use rand::Rng;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

pub const VALID_INPUT_CHARS: &str = "0123456789d.+-*/%() ";

#[derive(Clone)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollInformation {
    pub value: Decimal,
    pub processed_string: String,
    pub original_roll_texts: Vec<String>,
    pub rolls: Vec<Vec<Decimal>>,
}

impl RollInformation {
    fn new(value: Decimal, processed_string: String, original_roll_texts: Vec<String>, rolls: Vec<Vec<Decimal>>) -> RollInformation {
        RollInformation {
            value,
            processed_string,
            original_roll_texts,
            rolls,
        }
    }
}

#[derive(Clone)]
enum BinopSequenceMember {
    NonBinop(RollInformation),
    Binop(Binop),
}

#[derive(Parser)]
#[grammar = "dice.pest"]
struct DiceParser;

//////////////////////////
//   Helper functions   //
//////////////////////////

fn roll_die(sides: i128) -> Decimal {
    let roll = rand::thread_rng().gen_range(1..=sides);
    Decimal::from(roll)
}

fn roll_dice(number: Decimal, sides: Decimal) -> Result<(Decimal, Vec<Decimal>), String> {
    if number != number.floor() {
        return Err(String::from("Attempted to roll non-integer number of dice."));
    } else if sides != sides.floor() {
        return Err(String::from("Attempted to roll dice with non-integer number of sides."));
    } else if number.is_sign_negative() {
        return Err(String::from("Attempted to roll negative number of dice."));
    } else if !sides.is_sign_positive() {
        return Err(String::from("Attempted to roll dice with non-positive number of sides."));
    } else if number.is_zero() {
        return Ok((Decimal::from(0), Vec::new()));
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

    Ok((sum, rolls))
}

fn handle_binop_sequence_dice(mut sequence: Vec<BinopSequenceMember>) -> Result<Vec<BinopSequenceMember>, String> {
    let mut next_dice_check = sequence.iter().position(|member| match member {
        BinopSequenceMember::Binop(Binop::Dice) => true,
        _ => false,
    });

    while next_dice_check != None {
        let next_dice_position = next_dice_check.unwrap();
        let mut new_sequence = Vec::new();

        new_sequence.append(&mut sequence[..next_dice_position - 1].to_vec());
        match (sequence[next_dice_position - 1].clone(), sequence[next_dice_position + 1].clone()) {
            (BinopSequenceMember::NonBinop(info1), BinopSequenceMember::NonBinop(info2)) => {
                let (value, new_rolls) = roll_dice(info1.value, info2.value)?;
                let processed_string = String::from("{}");

                let mut original_roll_texts = info1.original_roll_texts.clone();
                original_roll_texts.push(format!("{}d{}", info1.processed_string, info2.processed_string));
                original_roll_texts.append(&mut info2.original_roll_texts.clone());

                let mut rolls = info1.rolls.clone();
                rolls.push(new_rolls);
                rolls.append(&mut info2.rolls.clone());

                new_sequence.push(BinopSequenceMember::NonBinop(RollInformation::new(value, processed_string, original_roll_texts, rolls)));
            }
            _ => panic!("Found binop(s) where non-binops were expected."),
        }
        if sequence.len() > next_dice_position + 2 {
            new_sequence.append(&mut sequence[next_dice_position + 2..].to_vec());
        }

        sequence = new_sequence;
        next_dice_check = sequence.iter().position(|member| match member {
            BinopSequenceMember::Binop(Binop::Dice) => true,
            _ => false,
        });
    }

    Ok(sequence)
}

fn handle_binop_sequence_times_divide_mod(mut sequence: Vec<BinopSequenceMember>) -> Result<Vec<BinopSequenceMember>, String> {
    let mut next_operator_check = sequence.iter().position(|member| match member {
        BinopSequenceMember::Binop(Binop::Times) | BinopSequenceMember::Binop(Binop::Divide) | BinopSequenceMember::Binop(Binop::Mod) => true,
        _ => false,
    });

    while next_operator_check != None {
        let next_operator_position = next_operator_check.unwrap();
        let next_operator = match sequence[next_operator_position].clone() {
            BinopSequenceMember::Binop(b) => b,
            _ => panic!("Found non-binop where binop was expected."),
        };
        let mut new_sequence = Vec::new();

        new_sequence.append(&mut sequence[..next_operator_position - 1].to_vec());
        match (sequence[next_operator_position - 1].clone(), sequence[next_operator_position + 1].clone()) {
            (BinopSequenceMember::NonBinop(info1), BinopSequenceMember::NonBinop(info2)) => {
                let (value, processed_string) = match next_operator {
                    Binop::Times => {
                        let value = info1.value * info2.value;
                        let processed_string = format!("{}*{}", info1.processed_string, info2.processed_string);
                        (value, processed_string)
                    }
                    Binop::Divide => {
                        let value = info1.value / info2.value;
                        let processed_string = format!("{}/{}", info1.processed_string, info2.processed_string);
                        (value, processed_string)
                    }
                    Binop::Mod => {
                        let value = info1.value % info2.value;
                        let processed_string = format!("{}%{}", info1.processed_string, info2.processed_string);
                        (value, processed_string)
                    }
                    _ => panic!("Found binop of incorrect type."),
                };

                let mut original_roll_texts = info1.original_roll_texts.clone();
                original_roll_texts.append(&mut info2.original_roll_texts.clone());

                let mut rolls = info1.rolls.clone();
                rolls.append(&mut info2.rolls.clone());

                new_sequence.push(BinopSequenceMember::NonBinop(RollInformation::new(value, processed_string, original_roll_texts, rolls)));
            }
            _ => panic!("Found binop(s) where non-binops were expected."),
        }
        if sequence.len() > next_operator_position + 2 {
            new_sequence.append(&mut sequence[next_operator_position + 2..].to_vec());
        }

        sequence = new_sequence;
        next_operator_check = sequence.iter().position(|member| match member {
            BinopSequenceMember::Binop(Binop::Times) | BinopSequenceMember::Binop(Binop::Divide) | BinopSequenceMember::Binop(Binop::Mod) => true,
            _ => false,
        });
    }

    Ok(sequence)
}

fn handle_binop_sequence_plus_minus(mut sequence: Vec<BinopSequenceMember>) -> Result<Vec<BinopSequenceMember>, String> {
    let mut next_operator_check = sequence.iter().position(|member| match member {
        BinopSequenceMember::Binop(Binop::Plus) | BinopSequenceMember::Binop(Binop::Minus) => true,
        _ => false,
    });

    while next_operator_check != None {
        let next_operator_position = next_operator_check.unwrap();
        let next_operator = match sequence[next_operator_position].clone() {
            BinopSequenceMember::Binop(b) => b,
            _ => panic!("Found non-binop where binop was expected."),
        };
        let mut new_sequence = Vec::new();

        new_sequence.append(&mut sequence[..next_operator_position - 1].to_vec());
        match (sequence[next_operator_position - 1].clone(), sequence[next_operator_position + 1].clone()) {
            (BinopSequenceMember::NonBinop(info1), BinopSequenceMember::NonBinop(info2)) => {
                let (value, processed_string) = match next_operator {
                    Binop::Plus => {
                        let value = info1.value + info2.value;
                        let processed_string = format!("{}+{}", info1.processed_string, info2.processed_string);
                        (value, processed_string)
                    }
                    Binop::Minus => {
                        let value = info1.value - info2.value;
                        let processed_string = format!("{}-{}", info1.processed_string, info2.processed_string);
                        (value, processed_string)
                    }
                    _ => panic!("Found binop of incorrect type."),
                };

                let mut original_roll_texts = info1.original_roll_texts.clone();
                original_roll_texts.append(&mut info2.original_roll_texts.clone());

                let mut rolls = info1.rolls.clone();
                rolls.append(&mut info2.rolls.clone());

                new_sequence.push(BinopSequenceMember::NonBinop(RollInformation::new(value, processed_string, original_roll_texts, rolls)));
            }
            _ => panic!("Found binop(s) where non-binops were expected."),
        }
        if sequence.len() > next_operator_position + 2 {
            new_sequence.append(&mut sequence[next_operator_position + 2..].to_vec());
        }

        sequence = new_sequence;
        next_operator_check = sequence.iter().position(|member| match member {
            BinopSequenceMember::Binop(Binop::Plus) | BinopSequenceMember::Binop(Binop::Minus) => true,
            _ => false,
        });
    }

    Ok(sequence)
}

fn handle_binop_sequence(sequence: Vec<BinopSequenceMember>) -> Result<RollInformation, String> {
    let dice_handled = handle_binop_sequence_dice(sequence)?;
    let times_divide_mod_handled = handle_binop_sequence_times_divide_mod(dice_handled)?;
    let plus_minus_handled = handle_binop_sequence_plus_minus(times_divide_mod_handled)?;

    if plus_minus_handled.len() != 1 {
        panic!("Incorrect length at the end of binop-sequence-handling.")
    } else if let BinopSequenceMember::NonBinop(info) = plus_minus_handled[0].clone() {
        Ok(info)
    } else {
        panic!("Incorrect type at the end of binop-sequence-handling.")
    }
}

pub fn clean_input(input: &str) -> String {
    let mut clean = String::from(input);
    clean.retain(|c| VALID_INPUT_CHARS.contains(c));

    clean
}

//////////////////////////
//   Parser functions   //
//////////////////////////

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
        _ => unreachable!("Non-binop found inside binop token."),
    }
}

fn parse_unop(unop: Pair<Rule>) -> Unop {
    assert_eq!(unop.as_rule(), Rule::unop, "Called parse_unop on non-unop.");

    let internal_unop = unop.into_inner().next().unwrap();

    match internal_unop.as_rule() {
        Rule::plus_unop => Unop::Plus,
        Rule::minus_unop => Unop::Minus,
        _ => unreachable!("Non-unop found inside unop token."),
    }
}

fn parse_paren_block(paren_block: Pair<Rule>) -> Result<RollInformation, String> {
    assert_eq!(paren_block.as_rule(), Rule::paren_block, "Called parse_paren_block on non-paren-block.");

    let mut inside = paren_block.into_inner();
    let mut binop_sequence_vec = Vec::new();
    let information = parse_legitimate_sequence(inside.next().unwrap())?;
    binop_sequence_vec.push(BinopSequenceMember::NonBinop(information));

    let mut next = inside.next();
    while next != None {
        let next_binop = parse_binop(next.unwrap());
        binop_sequence_vec.push(BinopSequenceMember::Binop(next_binop));

        let next_information = parse_legitimate_sequence(inside.next().unwrap())?;
        binop_sequence_vec.push(BinopSequenceMember::NonBinop(next_information));

        next = inside.next();
    }

    let handled = handle_binop_sequence(binop_sequence_vec)?;
    Ok(RollInformation::new(handled.value, format!("({})", handled.processed_string), handled.original_roll_texts, handled.rolls))
}

fn parse_non_operator(non_operator: Pair<Rule>) -> Result<RollInformation, String> {
    assert_eq!(non_operator.as_rule(), Rule::non_operator, "Called parse_non_operator on non-paren-block.");

    let inside = non_operator.into_inner().next().unwrap();

    match inside.as_rule() {
        Rule::number => {
            let (number, string) = parse_number(inside);
            Ok(RollInformation::new(number, string, Vec::new(), Vec::new()))
        }
        Rule::paren_block => parse_paren_block(inside),
        _ => unreachable!("Non-operator token inside isn't a number or paren block."),
    }
}

fn parse_paired_unop(paired_unop: Pair<Rule>) -> Result<RollInformation, String> {
    assert_eq!(paired_unop.as_rule(), Rule::paired_unop, "Called parse_paired_unop on non-paired-unop.");

    let mut inside = paired_unop.into_inner();

    let unop = inside.next().unwrap();
    let non_op = parse_non_operator(inside.next().unwrap())?;

    match parse_unop(unop) {
        Unop::Plus => Ok(RollInformation::new(non_op.value, format!("+{}", non_op.processed_string), non_op.original_roll_texts, non_op.rolls)),
        Unop::Minus => Ok(RollInformation::new(
            non_op.value * Decimal::from(-1),
            format!("-{}", non_op.processed_string),
            non_op.original_roll_texts,
            non_op.rolls,
        )),
    }
}

fn parse_non_binop(non_binop: Pair<Rule>) -> Result<RollInformation, String> {
    assert_eq!(non_binop.as_rule(), Rule::non_binop, "Called parse_non_binop on non-non-binop.");

    let inside = non_binop.into_inner().next().unwrap();

    match inside.as_rule() {
        Rule::number => {
            let (number, string) = parse_number(inside);
            Ok(RollInformation::new(number, string, Vec::new(), Vec::new()))
        }
        Rule::paren_block => parse_paren_block(inside),
        Rule::paired_unop => parse_paired_unop(inside),
        _ => unreachable!("Non-binop token inside isn't a number, paren block, or paired unop."),
    }
}

fn parse_legitimate_sequence(sequence: Pair<Rule>) -> Result<RollInformation, String> {
    assert_eq!(sequence.as_rule(), Rule::legitimate_sequence, "Called parse_legitimate_sequence on non-legitimate-sequence.");

    let mut inside = sequence.into_inner();
    let mut binop_sequence_vec = Vec::new();
    let information = parse_non_binop(inside.next().unwrap())?;
    binop_sequence_vec.push(BinopSequenceMember::NonBinop(information));

    let mut next = inside.next();
    while next != None {
        let next_binop = parse_binop(next.unwrap());
        binop_sequence_vec.push(BinopSequenceMember::Binop(next_binop));

        let next_information = parse_non_binop(inside.next().unwrap())?;
        binop_sequence_vec.push(BinopSequenceMember::NonBinop(next_information));

        next = inside.next();
    }

    Ok(handle_binop_sequence(binop_sequence_vec)?)
}

fn parse_full_expression(mut tree: Pairs<Rule>) -> Result<RollInformation, String> {
    let full_expression = tree.next().unwrap();
    let sequence = full_expression.into_inner().next().unwrap();

    parse_legitimate_sequence(sequence)
}

pub fn parse_input(input: &str) -> Result<RollInformation, String> {
    let cleaned = clean_input(input);
    match DiceParser::parse(Rule::full_expression, &cleaned) {
        Ok(full_expression) => Ok(parse_full_expression(full_expression)?),
        Err(e) => Err(e.to_string()),
    }
}
