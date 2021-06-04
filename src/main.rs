#![windows_subsystem = "windows"]

use rust_decimal::prelude::*;
use rand::Rng;
use iced::{Sandbox, Element, Column, Row, Text, TextInput, text_input, Button, button, Settings};

const NUMBERS: &str = "0123456789.";
const DIE: &str = "d";
const BINARY_OPERATORS: &str = "+-*/%";
const UNARY_OPERATORS: &str = "+-";
const PARENS: &str = "()";
const WHITESPACE: &str = " ";
const ALL_LEGAL_CHARACTERS: &str = "0123456789.d+-*/%+-() ";
// TODO: all_legal_characters is defined awkwardly. Figure out how to define a const &str as a concatenation of other const &strs.

const NUMERALS: &str = "0123456789"; // Used in number and die tokenization

/////////////////////////
/// Structs and Enums ///
/////////////////////////

#[derive(Clone)]
struct Dice {
    number: u128,
    sides: i64,
}

#[derive(Clone)]
enum Token {
    Number(Decimal),
    Dice(Dice),
    Operator(char), // TODO: separate binops and unops into separate token types, parsed differently from one another
    ParenBlock(String),
}

struct SyntaxTree {
    contents: Token,
    left: Option<Box<SyntaxTree>>,
    right: Option<Box<SyntaxTree>>,
}

////////////////////
/// Dice-Rolling ///
////////////////////

// TODO: figure out if there's a way to do RNG with an i128 or u128 instead, to minimize the risk of a number going out of range
fn roll_die(sides: i64) -> Decimal {
    // Expects die with at least 1 side.
    let roll = Decimal::from(rand::thread_rng().gen_range(1..(sides + 1)));

    roll
}

// fn roll_dice(dice_to_roll: Dice) -> Decimal {
//     let mut roll_total = Decimal::from(0);
//     for _ in 0..dice_to_roll.number {
//         let roll = roll_die(dice_to_roll.sides);
//         roll_total += roll;
//     }

//     roll_total
// }

fn roll_dice_verbose(dice_to_roll: Dice) -> (Decimal, String) {
    let mut roll_total = Decimal::from(0);
    let mut roll_string = String::from("(");
    for _ in 0..dice_to_roll.number {
        let roll = roll_die(dice_to_roll.sides);
        roll_total += roll;

        roll_string.push_str(&format!("{}+", roll));
    }

    let string_length = roll_string.len();

    let mut final_string: String;
    if string_length == 1 {
        final_string = String::from("(0");
    } else {
        final_string = String::from(&roll_string[..string_length - 1]);
        // Safe because plus is a known one-byte character
    }
    final_string.push(')');

    (roll_total, final_string)
}

////////////////////
/// Tokenization ///
////////////////////

fn sanitize_string_for_tokenization (string_to_sanitize: String) -> Result<String, &'static str> {
    let mut sanitized_string = String::new();

    for character in string_to_sanitize.chars() {
        if ALL_LEGAL_CHARACTERS.contains(character)
        && !WHITESPACE.contains(character) {
            sanitized_string.push(character);
        } else if !ALL_LEGAL_CHARACTERS.contains(character) {
            return Err("Input contains illegal character. Please input only numerals, parentheses, decimal points, the letter 'd', the legal operators +, -, *, /, and %, and spaces.");
        }
    }

    Ok(sanitized_string)
}

fn get_paren_block_token(vector_being_tokenized: Vec<char>)
-> Result<(Token, usize), &'static str> {
    // Expects input to start with a paren.

    let mut left_paren_count = 0;
    let mut right_paren_count = 0;
    let mut token = Token::ParenBlock(String::from("")); // TODO: token should ideally be uninitialized, but the compiler didn't recognize the below while loop as always initializing it, so this early initialization was necessary to compile. Figure out if there's an alternative that lets it stay more sensibly uninitialized here without dipping into Unsafe territory.

    let mut current_index = 0;
    let max_index = vector_being_tokenized.len();

    while current_index < max_index {
        let current_char = vector_being_tokenized[current_index];

        match current_char {
            '(' => {
                left_paren_count += 1;
            }
            ')' => {
                right_paren_count += 1;

                if right_paren_count == left_paren_count {
                    let paren_block: String = 
                        vector_being_tokenized[..(current_index + 1)]
                        .iter().collect();
                    token = Token::ParenBlock(paren_block);
                    break;

                } else if right_paren_count > left_paren_count {
                    return Err("Paren closed without opening.");
                }
            }
            _ => {} // If current_char is anything else, it's inside the parens and thus not worth worrying about here.
        }

        current_index += 1;
    }

    if current_index == max_index {return Err("Paren opened without closing.")}

    let next_index = current_index + 1;
    Ok((token, next_index))
}

fn get_number_token(vector_being_tokenized: Vec<char>)
-> Result<(Token, usize), &'static str> {
    // Expects input to start with a numeral or a '.'.

    let mut numerals_before_decimal = false;
    let result: Decimal;

    let mut before_decimal_string = String::new();
    let mut after_decimal_string = String::new();

    let mut current_index = 0;
    let max_index = vector_being_tokenized.len();

    while current_index < max_index && // Get initial numeral string
    NUMERALS.contains(vector_being_tokenized[current_index]) {
        numerals_before_decimal = true;
        before_decimal_string.push(vector_being_tokenized[current_index]);

        current_index += 1;
    }

    if current_index == max_index && numerals_before_decimal { // Reached end of input, and got a numeral first; return.
        result = Decimal::from_str(&before_decimal_string).unwrap();
        // Safe because, if numerals_before_decimal, then before_decimal_string contains at least one numeral and no non-numerals.

    } else if current_index == max_index && !numerals_before_decimal {
        return Err("Input contains decimal point without any numbers before or after.");

    } else if !NUMBERS.contains(vector_being_tokenized[current_index]) { // The next character is a non-number non-numeral, and thus not a decimal point; therefore, the input contains no decimal points. Return the number.
        result = Decimal::from_str(&before_decimal_string).unwrap();

    } else if vector_being_tokenized[current_index] == '.' {
        current_index += 1;

        while current_index < max_index && vector_being_tokenized[current_index] == '.' { // Assume that multiple decimals in a row are typos and skip past them
            current_index += 1;
        }

        if current_index == max_index || !NUMBERS.contains(vector_being_tokenized[current_index]) {
            if numerals_before_decimal { // Assume 'x.' to be intended as equivalent to 'x'.
                result = Decimal::from_str(&before_decimal_string).unwrap();
                // Safe given numerals_before_decimal, as above
            } else {
                return Err("Input contains decimal point without any numbers before or after.");
            }

        } else { //Capture the following string of numerals
            while current_index < max_index &&
                NUMERALS.contains(vector_being_tokenized[current_index]) {
                    after_decimal_string.push
                        (vector_being_tokenized[current_index]);
                    current_index += 1;
                }
            
            if current_index == max_index || !NUMBERS.contains(vector_being_tokenized[current_index]) { // Successful capture
                if numerals_before_decimal { // Number is of form 'x.y'
                    result = Decimal::from_str(&format!(
                        "{}.{}", before_decimal_string, after_decimal_string)
                        ).unwrap();
                    // Safe given prior algorithm preventing non-numerals from entering the strings
                } else { // Number is of form '.x'
                    result = Decimal::from_str
                        (&format!("0.{}", after_decimal_string)).unwrap();
                    // Safe, as in the previous branch
                }

            } else { // There's another decimal where there shouldn't be
                return Err("Input contains more than one separate decimal point.");
            }
        }
    } else {
        return Err("Something weird happened in number tokenization. (This error should never be returned; if you're seeing it, please submit a bug report containing this error and the input which led to it.)");
        // This should never happen unless the function's assumptions about its input are violated.
    }

    let token = Token::Number(result);
    Ok((token, current_index))
}

fn get_dice_token(vector_being_tokenized: Vec<char>)
-> Result<(Token, usize), &'static str> {
    // Expects input to start with a numeral or a 'd', and to contain a 'd' before any operators, parens, or the end of the input vec.

    let mut numbers_before_die = false;
    let token: Token;

    let mut before_die_string = String::new();
    let mut after_die_string = String::new();

    let mut current_index = 0;
    let max_index = vector_being_tokenized.len();

    while current_index < max_index && // Get initial number string
    NUMBERS.contains(vector_being_tokenized[current_index]) {
        numbers_before_die = true;
        before_die_string.push(vector_being_tokenized[current_index]);

        current_index += 1;
    }

    if current_index == max_index
    || !DIE.contains(vector_being_tokenized[current_index]) { // Assumptions were violated
        return Err("Attempted die tokenization on non-die input.");
    
    } else { // vector_being_tokenized[current_index] is a 'd'.
        while current_index < max_index
        && DIE.contains(vector_being_tokenized[current_index]) { // Assume that multiple 'd's in a row are typos and skip past them
            current_index += 1;
        }

        if current_index == max_index || !NUMBERS.contains(vector_being_tokenized[current_index]) {
            return Err("Dice input contains no number after the 'd'.");

        } else { //Get post-'d' number string
            while current_index < max_index &&
                NUMBERS.contains(vector_being_tokenized[current_index]) {
                    after_die_string.push
                        (vector_being_tokenized[current_index]);
                    current_index += 1;
                }
            
            if before_die_string.contains('.') {
                return Err("Attempted to roll a fractional number of dice.");
            } else if after_die_string.contains('.') {
                return Err("Attempted to roll dice with a fractional number of sides.");
            }

            let die_number: u128;
            if numbers_before_die {
                die_number = before_die_string.parse().unwrap(); // Safe due to prior filtering of string contents.
            } else {
                die_number = 1;
            }

            let die_sides: i64 = after_die_string.parse().unwrap(); // Safe due to prior filtering of string contents.

            let dice = Dice{number: die_number, sides: die_sides};
            token = Token::Dice(dice);
        }
    }

    Ok((token, current_index))
}

fn get_number_or_die_token(vector_being_tokenized: Vec<char>)
-> Result<(Token, usize), &'static str> {
    // Expects input to start with a numeral, '.', or 'd'.

    let mut is_dice = false;
    let mut current_index = 0;
    let max_index = vector_being_tokenized.len();

    while current_index < max_index
    && (NUMBERS.contains(vector_being_tokenized[current_index])
    || DIE.contains(vector_being_tokenized[current_index])) { // Traverse in search of a 'd'.
        if DIE.contains(vector_being_tokenized[current_index]) {
            is_dice = true;
            break;
        }
        current_index += 1;
    }

    let final_result: Result<(Token, usize), &'static str>;
    if is_dice {
        final_result = get_dice_token(vector_being_tokenized);
    } else {
        final_result = get_number_token(vector_being_tokenized);
    }

    final_result
}

fn get_unop_token(vector_being_tokenized: Vec<char>)
-> Result<(Token, usize), &'static str> {
    // Expects input to start with a unary operator
    // Returns ParenBlock token of (0 [unop] [input to unop])

    let operator = vector_being_tokenized[0];
    let mut unop_paren_string = String::from(format!("(0{}", operator));
    // First two parts of the aforementioned ParenBlock, the 0 and the operator

    let mut current_index = 1;

    if vector_being_tokenized.len() == 1 {
        return Err("Found unary operator at end of input.")
    } else {
        let after_operator = vector_being_tokenized[1];

        if NUMBERS.contains(after_operator) // Finish output ParenBlock for number or die
        || DIE.contains(after_operator) {
            let after_operator_tokenization_result = get_number_or_die_token 
                (vector_being_tokenized[1..].to_vec())?;

            current_index += after_operator_tokenization_result.1;

            let after_operator_token = after_operator_tokenization_result.0;
            match after_operator_token {
                Token::Number(num) => {
                    unop_paren_string.push_str(&format!("{})", num));
                }
                Token::Dice(dice) => {
                    unop_paren_string.push_str
                        (&format!("{}d{})", dice.number, dice.sides));
                }
                _ => {
                    return Err("Non-number-or-die received from attempted number or die tokenization. (This error should never be returned; if you're seeing it, please submit a bug report containing this error and the input which led to it.)")
                    // Should never happen if number-or-die tokenization goes correctly.
                }
            }

        } else if PARENS.contains(after_operator) { // Finish output ParenBlock for paren block
            let after_operator_tokenization_result = get_paren_block_token
                (vector_being_tokenized[1..].to_vec())?;
                
            current_index += after_operator_tokenization_result.1;

            if let Token::ParenBlock(block) = after_operator_tokenization_result.0 {
                unop_paren_string.push_str(&format!("{})", block));
            }

        } else if UNARY_OPERATORS.contains(after_operator) { // Finish output ParenBlock for nested unary operator
            let after_operator_tokenization_result = get_unop_token
                (vector_being_tokenized[1..].to_vec())?;

            current_index += after_operator_tokenization_result.1;

            if let Token::ParenBlock(block) = after_operator_tokenization_result.0 {
                unop_paren_string.push_str(&format!("{})", block));
            }

        } else if BINARY_OPERATORS.contains(after_operator) {
            return Err("Encountered unary operator with zero inputs.")
        }
    }

    let token = Token::ParenBlock(unop_paren_string);

    Ok((token, current_index))
}

fn tokenize(string_to_tokenize: String)
-> Result<Box<Vec<Token>>, &'static str> {
    // Expected format: a non-binary-operator token, followed by 0 or more [binary operator, non-binary-operator] sequences.
    let mut tokens: Vec<Token> = Vec::new();

    let string_chars: Vec<char> = string_to_tokenize.chars().collect();
    
    let mut current_index = 0;
    let max_index = string_chars.len();

    while current_index < max_index {
        // First major loop-chunk: get number, die roll, or paren-block (including unary operators)
        let mut current_char = string_chars[current_index];
        
        let tokenization_result: (Token, usize);

        if PARENS.contains(current_char) { // current_char is a paren-block
            tokenization_result = get_paren_block_token
                (string_chars[current_index..].to_vec())?;

        } else if NUMBERS.contains(current_char)
        || DIE.contains(current_char) { // current_char is a number or a die
            tokenization_result = get_number_or_die_token
                (string_chars[current_index..].to_vec())?;
        
        } else if UNARY_OPERATORS.contains(current_char) { // current_char is a unary operator (tokenized as a paren-block token)
            tokenization_result = get_unop_token
                (string_chars[current_index..].to_vec())?;

        } else { // current_char is a binary operator
            return Err("Found binary operator where non-binary-operator was expected.")
        }

        tokens.push(tokenization_result.0);
        current_index += tokenization_result.1;

        // Second major loop-chunk: Reach end of (well-formed) string, or get operator.
        if current_index == max_index {break}
        else {current_char = string_chars[current_index]};
        if BINARY_OPERATORS.contains(current_char) {
            let operator_token = Token::Operator(current_char);
            tokens.push(operator_token);
            current_index += 1;
            if current_index == max_index{
                return Err("Final token is a two-place operator with only one place filled.");
            }
        } else {
            return Err("Encountered two non-binary-operator syntactic units in a row.");
        }
    }

    if tokens.len() == 0 {
        return Err("Couldn't render sanitized input into tokens.");
    }

    Ok(Box::new(tokens))
}

/////////////////////
/// Tree-Building ///
/////////////////////

fn tokenize_paren_block(paren_block: String)
-> Result<Box<Vec<Token>>, &'static str> {
    let paren_block_literal = &paren_block;
    let trimmed_literal = &paren_block[1..(paren_block_literal.len() - 1)];
    // The trim is safe because the characters on the ends of the block are known to be parens, and thus one-byte characters.
    let trimmed_paren_block = String::from(trimmed_literal);

    tokenize(trimmed_paren_block)
}

fn parse_tokens_to_tree(tokens: Box<Vec<Token>>)
-> Result<SyntaxTree, &'static str> {
    let mut parsed_tree: SyntaxTree;

    let mut current_token = tokens[0].clone();

    // Pre-loop preparation: parse the first token into the leftmost part of the tree
    match current_token {
        Token::Number(_) | Token::Dice(_) => {
            parsed_tree = SyntaxTree{
                contents: current_token,
                left: None,
                right: None,
            };
        }
        Token::ParenBlock(paren_block) => {
            let tokenized_block = tokenize_paren_block(paren_block)?;
            parsed_tree = parse_tokens_to_tree(tokenized_block)?;
        }
        Token::Operator(_) => {
            return Err("Ill-formed tokens; first token is an operator. (This error should never be returned; if you're seeing it, please submit a bug report containing this error and the input which led to it.)");
            // Should never happen if tokenization goes correctly.
        }
    }
    
    let highest_token_index = tokens.len();
    let mut current_token_index = 1;
    while current_token_index < highest_token_index {
        // Assumption: tokens will be well-formed. Odd number of elements, alternating [number | dice | paren] and operators.

        // First part: expand the tree with the next operator.
        current_token = tokens[current_token_index].clone();
        current_token_index += 1;

        if let Token::Operator(_) = current_token {
            parsed_tree = SyntaxTree {
                contents: current_token,
                left: Some(Box::new(parsed_tree)),
                right: None,
            }
        } else {
            return Err("Ill-formed tokens; failed to find binary operator where expected. (This error should never be returned; if you're seeing it, please submit a bug report containing this error and the input which led to it.)")
            // Should never happen if tokenization goes correctly.
        }

        // Second part: get the next [number | dice | paren]
        current_token = tokens[current_token_index].clone();
        current_token_index += 1;

        match current_token {
            Token::Number(_) | Token::Dice(_) => {
                parsed_tree.right = Some(Box::new(SyntaxTree{
                    contents: current_token,
                    left: None,
                    right: None,
                }))
            }
            Token::ParenBlock(paren_block) => {
                let tokenized_block = tokenize_paren_block(paren_block)?;
                let resulting_tree = parse_tokens_to_tree(tokenized_block)?;
                parsed_tree.right = Some(Box::new(resulting_tree));
            }
            Token::Operator(_) => {
                return Err("Ill-formed tokens; operator found where non-operator expected. (This error should never be returned; if you're seeing it, please submit a bug report containing this error and the input which led to it.)");
                // Should never happen if tokenization goes correctly.
            }
        
        // End of loop: if the tokens are finished then break, otherwise do another round
        }
    }

    Ok(parsed_tree)
}

//////////////////////
/// Tree Traversal ///
//////////////////////

fn traverse_and_roll_verbose(tree: SyntaxTree) -> (Decimal, Vec<String>) {
    let result: Decimal;
    let mut rolls = Vec::<String>::new();

    if tree.left.is_none() & tree.right.is_none() {
        match tree.contents {
            Token::Number(number_decimal) => {
                result = number_decimal;
            }
            Token::Dice(dice_struct) => {
                let roll = roll_dice_verbose(dice_struct);
                result = roll.0;
                rolls.push(roll.1);
            }
            Token::Operator(_) => {
                panic!("Ill-formed tree; contains operator without children. (This error should never be returned; if you're seeing it, please submit a bug report containing this error and the input which led to it.)")
                // Should never happen if tokenization and tree-building go correctly.
            }
            Token::ParenBlock(_) => {
                panic!("Ill-formed tree; contains untokenized paren block. (This error should never be returned; if you're seeing it, please submit a bug report containing this error and the input which led to it.)")
                // Should never happen if tree-building goes correctly.
            }
        }

    } else if (tree.left.is_none() & !tree.right.is_none())
        || (!tree.left.is_none() & tree.right.is_none()) {
            panic!("Ill-formed tree; contains parent with only one child. (This error should never be returned; if you're seeing it, please submit a bug report containing this error and the input which led to it.)")
            // Should never happen if tokenization and tree-building go correctly.

    } else {
        if let Token::Operator(op) = tree.contents {
            let mut left = traverse_and_roll_verbose(*tree.left.unwrap());
            let mut right = traverse_and_roll_verbose(*tree.right.unwrap());
            // Both of the above are safe because, if either were None, they'd have been filtered out in the prior branches.
            
            rolls.append(&mut left.1);
            rolls.append(&mut right.1);
            
            match op {
                '+' => result = left.0 + right.0,
                '-' => result = left.0 - right.0,
                '*' => result = left.0 * right.0,
                '/' => result = left.0 / right.0,
                '%' => result = left.0 % right.0,
                _ => panic!("Non-operator contained in Operator token. (This error should never be returned; if you're seeing it, please submit a bug report containing this error and the input which led to it.)"),
                // Should never happen if tokenization goes correctly.
            }
        } else {
            panic!("Ill-formed tree; contains non-operator with children. (This error should never be returned; if you're seeing it, please submit a bug report containing this error and the input which led to it.)")
            // Should never happen if tokenization and tree-building go correctly.
        }
    }

    (result, rolls)
}

////////////////
/// Frontend ///
////////////////

fn tokens_and_rolls_to_string(tokens: Vec<Token>, rolls: Vec<String>)
-> Result<(String, usize), &'static str> {
    // Expects rolls to have a number of members equal or greater to the number of Dice tokens in tokens
    let mut result_string = String::new();

    let mut roll_counter = 0;
    for token in tokens {
        match token{
            Token::Number(num) => {
                result_string.push_str(&format!("{}", num));
            }
            Token::Dice(_) => {
                result_string.push_str(&rolls[roll_counter]);
                roll_counter += 1;
            }
            Token::Operator(op) => {
                result_string.push_str(&format!("{}", op));
            }
            Token::ParenBlock(block) => {
                let tokenized_block = tokenize_paren_block(block)?;

                let paren_handling_results = tokens_and_rolls_to_string 
                    (*tokenized_block, rolls[roll_counter..].to_vec())?;

                roll_counter += paren_handling_results.1;
                
                result_string.push('(');
                result_string.push_str(&paren_handling_results.0);
                result_string.push(')');
            }
        }
    }

    Ok((result_string, roll_counter))
}

fn verbose_roll_from_string(raw_input: String)
-> Result<(String, Decimal), &'static str> {
    let sanitized = sanitize_string_for_tokenization(raw_input)?;
    let tokenized = tokenize(sanitized)?;
    let syntaxtree = parse_tokens_to_tree(tokenized.clone())?;

    let (result, rolls) = traverse_and_roll_verbose(syntaxtree);
    let rolls_verbose = tokens_and_rolls_to_string(*tokenized, rolls)?.0;

    Ok((rolls_verbose, result))
}

fn main_run(input: String) -> String {
    // loop {
        // print!("Please input your roll, or type 'exit' to exit.\n> ");
        // io::stdout().flush().unwrap();
        // Unsafe; should be improved in the final product

        // let mut user_input = String::new();
        // io::stdin()
        //     .read_line(&mut user_input)
        //     .expect("Failed to read input.");
        //     // Unsafe; should be improved in the final product
        
        // if input.trim().eq("exit") {return String::from("Exiting.")}//{break}

        let results = verbose_roll_from_string(input.clone());

        match results {
            Err(error) => {
                // println!("Input: {}\nError: {}\n", user_input.trim(), error);
                format!("Input: {}\nError: {}\n", input.trim(), error)
            }
            Ok(contents) => {
                // println!("Input: {}\nRolled: {}\nResult: {}\n",
                    // user_input.trim(), contents.0, contents.1);
                format!("Input: {}\nRolled: {}\nResult: {}\n",
                    input.trim(), contents.0, contents.1)
            }
        }
    // }
}

/////////////
//   GUI   //
/////////////

#[derive(Debug, Clone)]
enum Message {
    InputAppend(String),
    InputReady,
}

struct Calculator {
    current_input: String,
    latest_output: String,
    text_input_state: text_input::State,
    ready_button_state: button::State,
}

impl Sandbox for Calculator {
    type Message = Message;

    fn new() -> Calculator {
        Calculator{
            current_input: String::new(),
            latest_output: String::new(),
            text_input_state: text_input::State::new(),
            ready_button_state: button::State::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Dice Roller Which Needs A Better Name Before Release")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::InputAppend(c) => {
                if ALL_LEGAL_CHARACTERS.contains(&c) { // Automatically sanitize inputs as we go
                    self.current_input.push_str(&c);
                }
            }
            Message::InputReady => {
                self.latest_output = main_run(self.current_input.clone());
                self.current_input = String::new();
            }
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        Column::new()
            .padding(20)
            .push(
                TextInput::new(
                    &mut self.text_input_state,
                    &self.current_input,
                    "",
                    Self::Message::InputAppend
                )
                .size(50)
                .on_submit(Self::Message::InputReady)
            )
            .push(
                Button::new(
                    &mut self.ready_button_state,
                        Text::new("Roll"))
                    .on_press(Self::Message::InputReady)
            )
            .push(Text::new(&self.latest_output).size(50))
            .into()
    }
}

//////////////
//   Main   //
//////////////

fn main() {
    Calculator::run(Settings::default()).unwrap();
}

// Extremely long-term TODO: completely restructure the back-end syntax parsing to as to allow paren blocks which evaluate to int-equivalent values to serve as inputs to dice rolls. (This essentially means changing dice rolls to basically-binops, except with different syntactic rules, including non-greedy input-grabbing. It'll be a horrible mess and require massive revisions to nearly every part of the code here. Implement it only when I have nothing else left to do, or at least when everything left is comparably impractical.)