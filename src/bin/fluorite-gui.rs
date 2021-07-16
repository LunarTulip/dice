#![windows_subsystem = "windows"]

use druid::keyboard_types::Key;
use druid::text::format::{Formatter, Validation, ValidationError};
use druid::text::selection::Selection;
use druid::widget::prelude::*;
use druid::widget::{Align, Button, Controller, Flex, Label, List, Scroll, SizedBox, TextBox, ValueTextBox};
use druid::{AppLauncher, Command, Data, Lens, LocalizedString, MenuDesc, MenuItem, Selector, Target, Widget, WidgetExt, WindowDesc};
use fluorite::format_string_with_results;
use fluorite::parse::{clean_input, parse_input, RollInformation, VALID_INPUT_CHARS};
use std::error::Error;
use std::sync::Arc;

/////////////////
//   Structs   //
/////////////////

enum CalcButton {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Decimal,
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    Plus,
    Minus,
    Times,
    Divide,
    Mod,
    Dice,
    OpenParen,
    CloseParen,
    ClearEntry,
    Clear,
    Backspace,
    Roll,
    PlaceholderNotCurrentlyInUse,
}

#[derive(Clone, Data)]
struct RollShortcut {
    name: String,
    roll: String,
}

#[derive(Clone, Data, Lens)]
struct DiceCalculator {
    current_input: String,
    stored_input: String,
    history: Arc<Vec<(String, Result<RollInformation, String>)>>,
    steps_back_in_history: usize,
    shortcuts: Arc<Vec<RollShortcut>>,
    new_shortcut_name: String,
    new_shortcut_text: String,
}

impl DiceCalculator {
    fn new() -> DiceCalculator {
        DiceCalculator {
            current_input: String::new(),
            stored_input: String::new(),
            history: Arc::new(Vec::new()),
            steps_back_in_history: 0,
            shortcuts: Arc::new(Vec::new()),
            new_shortcut_name: String::new(),
            new_shortcut_text: String::new(),
        }
    }
    fn roll(&mut self) {
        Arc::make_mut(&mut self.history).push((self.current_input.clone(), parse_input(&self.current_input)));
        self.current_input = String::new();
        self.stored_input = String::new();
        self.steps_back_in_history = 0;
    }
    fn roll_from_shortcut(&mut self, shortcut: RollShortcut) {
        Arc::make_mut(&mut self.history).push((shortcut.roll.clone(), parse_input(&shortcut.roll)));
        if self.steps_back_in_history != 0 {
            self.current_input = self.stored_input.clone()
        }
        self.steps_back_in_history = 0;
    }
    fn add_shortcut(_ctx: &mut EventCtx, data: &mut Self, _env: &Env) {
        let new_shortcut = RollShortcut {
            name: data.new_shortcut_name.clone(),
            roll: data.new_shortcut_text.clone(),
        };
        if !data.shortcuts.iter().any(|shortcut| shortcut.name == new_shortcut.name || new_shortcut.name == "") {
            Arc::make_mut(&mut data.shortcuts).insert(0, new_shortcut);
            data.new_shortcut_name = String::new();
            data.new_shortcut_text = String::new();
        } // Figure out some way to provide clear feedback on success/failure
    }
}

struct DiceCalcEventHandler;

impl<W: Widget<DiceCalculator>> Controller<DiceCalculator, W> for DiceCalcEventHandler {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut DiceCalculator, env: &Env) {
        match event {
            Event::WindowConnected => ctx.request_focus(),
            Event::MouseDown(_) => ctx.request_focus(),
            Event::KeyDown(key_event) if ctx.is_focused() => match &key_event.key {
                Key::Character(s) => {
                    if VALID_INPUT_CHARS.contains(s) {
                        data.current_input.push_str(&s);
                    }
                }
                Key::Backspace => {
                    let _ = data.current_input.pop();
                }
                Key::ArrowUp => {
                    let history_len = data.history.len();
                    if data.steps_back_in_history < history_len {
                        if data.steps_back_in_history == 0 {
                            data.stored_input = data.current_input.clone();
                        }
                        data.steps_back_in_history += 1;
                        data.current_input = data.history[history_len - data.steps_back_in_history].0.clone();
                    }
                }
                Key::ArrowDown => {
                    if data.steps_back_in_history > 0 {
                        data.steps_back_in_history -= 1;
                        data.current_input = if data.steps_back_in_history == 0 {
                            data.stored_input.clone()
                        } else {
                            data.history[data.history.len() - data.steps_back_in_history].0.clone()
                        }
                    }
                }
                Key::Enter => data.roll(),
                _ => (),
            },
            Event::Command(command) => {
                if command.is::<RollShortcut>(Selector::new("ShortcutRoll")) {
                    let shortcut = command.get_unchecked::<RollShortcut>(Selector::new("ShortcutRoll")).clone();
                    data.roll_from_shortcut(shortcut);
                } else if command.is::<RollShortcut>(Selector::new("ShortcutDelete")) {
                    let name_to_delete = command.get_unchecked::<RollShortcut>(Selector::new("ShortcutDelete")).name.clone();
                    Arc::make_mut(&mut data.shortcuts).retain(|shortcut| shortcut.name != name_to_delete);
                }
            }
            _ => (),
        }
        child.event(ctx, event, data, env);
    }
}

#[derive(Debug)]
struct FormatValidationError;

impl std::fmt::Display for FormatValidationError {
    // Ugly hack; build a real implementation.
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Error for FormatValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

struct DiceTextFormatter;

impl Formatter<String> for DiceTextFormatter {
    fn format(&self, value: &String) -> String {
        value.clone()
    }
    fn validate_partial_input(&self, input: &str, _sel: &Selection) -> Validation {
        let input_cleaned = clean_input(input);
        if &input_cleaned == input {
            Validation::success()
        } else {
            Validation::failure(FormatValidationError {}).change_text(input_cleaned)
        }
    }
    fn value(&self, input: &str) -> Result<String, ValidationError> {
        Ok(String::from(input))
    }
}

//////////////////////
//   GUI Assembly   //
//////////////////////

fn build_calc_button(button: CalcButton, label: &str) -> impl Widget<DiceCalculator> {
    SizedBox::new(Label::new(label).center())
        .width(100.)
        .height(100.)
        .on_click(move |_ctx, data: &mut DiceCalculator, _env| match button {
            CalcButton::Zero => data.current_input.push('0'),
            CalcButton::One => data.current_input.push('1'),
            CalcButton::Two => data.current_input.push('2'),
            CalcButton::Three => data.current_input.push('3'),
            CalcButton::Four => data.current_input.push('4'),
            CalcButton::Five => data.current_input.push('5'),
            CalcButton::Six => data.current_input.push('6'),
            CalcButton::Seven => data.current_input.push('7'),
            CalcButton::Eight => data.current_input.push('8'),
            CalcButton::Nine => data.current_input.push('9'),
            CalcButton::Decimal => data.current_input.push('.'),
            CalcButton::D4 => (),  // TODO
            CalcButton::D6 => (),  // TODO
            CalcButton::D8 => (),  // TODO
            CalcButton::D10 => (), // TODO
            CalcButton::D12 => (), // TODO
            CalcButton::D20 => (), // TODO
            CalcButton::Plus => data.current_input.push('+'),
            CalcButton::Minus => data.current_input.push('-'),
            CalcButton::Times => data.current_input.push('*'),
            CalcButton::Divide => data.current_input.push('/'),
            CalcButton::Mod => data.current_input.push('%'),
            CalcButton::Dice => data.current_input.push('d'),
            CalcButton::OpenParen => data.current_input.push('('),
            CalcButton::CloseParen => data.current_input.push(')'),
            CalcButton::ClearEntry => (), // TODO
            CalcButton::Clear => data.current_input = String::new(),
            CalcButton::Backspace => {
                let _ = data.current_input.pop();
            }
            CalcButton::Roll => data.roll(),
            CalcButton::PlaceholderNotCurrentlyInUse => (),
        })
}

fn build_current_input_display() -> impl Widget<DiceCalculator> {
    Label::<DiceCalculator>::dynamic(|calc, _env| if calc.current_input.is_empty() { String::from("Roll text") } else { String::from(&calc.current_input) })
}

fn build_main_calculator_display() -> impl Widget<DiceCalculator> {
    Flex::column()
        .with_flex_child(
            Flex::row()
                .with_child(build_calc_button(CalcButton::D20, "d20"))
                .with_child(build_calc_button(CalcButton::ClearEntry, "CE"))
                .with_child(build_calc_button(CalcButton::Clear, "C"))
                .with_child(build_calc_button(CalcButton::Backspace, "[Backspace]"))
                .with_child(build_calc_button(CalcButton::Dice, "d")),
            1.,
        )
        .with_flex_child(
            Flex::row()
                .with_child(build_calc_button(CalcButton::D12, "d12"))
                .with_child(build_calc_button(CalcButton::OpenParen, "("))
                .with_child(build_calc_button(CalcButton::CloseParen, ")"))
                .with_child(build_calc_button(CalcButton::Mod, "%"))
                .with_child(build_calc_button(CalcButton::Divide, "[Divide]")),
            1.,
        )
        .with_flex_child(
            Flex::row()
                .with_child(build_calc_button(CalcButton::D10, "d10"))
                .with_child(build_calc_button(CalcButton::Seven, "7"))
                .with_child(build_calc_button(CalcButton::Eight, "8"))
                .with_child(build_calc_button(CalcButton::Nine, "9"))
                .with_child(build_calc_button(CalcButton::Times, "*")),
            1.,
        )
        .with_flex_child(
            Flex::row()
                .with_child(build_calc_button(CalcButton::D8, "d8"))
                .with_child(build_calc_button(CalcButton::Four, "4"))
                .with_child(build_calc_button(CalcButton::Five, "5"))
                .with_child(build_calc_button(CalcButton::Six, "6"))
                .with_child(build_calc_button(CalcButton::Minus, "-")),
            1.,
        )
        .with_flex_child(
            Flex::row()
                .with_child(build_calc_button(CalcButton::D6, "d6"))
                .with_child(build_calc_button(CalcButton::One, "1"))
                .with_child(build_calc_button(CalcButton::Two, "2"))
                .with_child(build_calc_button(CalcButton::Three, "3"))
                .with_child(build_calc_button(CalcButton::Plus, "+")),
            1.,
        )
        .with_flex_child(
            Flex::row()
                .with_child(build_calc_button(CalcButton::D4, "d4"))
                .with_child(build_calc_button(CalcButton::PlaceholderNotCurrentlyInUse, "[Placeholder]"))
                .with_child(build_calc_button(CalcButton::Zero, "0"))
                .with_child(build_calc_button(CalcButton::Decimal, "."))
                .with_child(build_calc_button(CalcButton::Roll, "[Roll]")),
            1.,
        )
}

fn build_main_column() -> impl Widget<DiceCalculator> {
    Flex::column().with_flex_child(build_current_input_display(), 1.).with_flex_child(build_main_calculator_display(), 1.)
}

fn build_latest_output_display() -> impl Widget<DiceCalculator> {
    Label::<DiceCalculator>::dynamic(|calc, _env| match calc.history.last() {
        None => String::new(),
        Some(roll_result) => match &roll_result.1 {
            Err(_) => String::from("ERROR"),
            Ok(info) => format!("{}", info.value),
        },
    })
}

fn build_history_display() -> impl Widget<DiceCalculator> {
    Label::<DiceCalculator>::dynamic(|calc, _| {
        let mut history = String::new();
        for roll_result in calc.history.iter().rev() {
            match roll_result {
                (input, Err(e)) => history.push_str(&format!("Input: {}\nError: {}\n\n", input, e)),
                (input, Ok(info)) => history.push_str(&format!(
                    "Input: {}\nRolled: {}\nResult: {}\n\n",
                    input,
                    format_string_with_results(&info.processed_string, info.rolls.clone()),
                    info.value
                )),
            }
        }
        history
    })
}

fn build_history_column() -> impl Widget<DiceCalculator> {
    Flex::column().with_flex_child(build_latest_output_display(), 1.).with_flex_child(Scroll::new(build_history_display()), 1.)
}

fn build_shortcut_creation_interface() -> impl Widget<DiceCalculator> {
    Flex::column()
        .with_flex_child(TextBox::new().with_placeholder("Name").lens(DiceCalculator::new_shortcut_name), 1.)
        .with_flex_child(ValueTextBox::new(TextBox::new().with_placeholder("Roll Text"), DiceTextFormatter {}).lens(DiceCalculator::new_shortcut_text), 1.)
        .with_flex_child(Button::new("Create Shortcut").on_click(DiceCalculator::add_shortcut), 1.)
}

fn build_shortcut_list() -> impl Widget<DiceCalculator> {
    List::new(|| {
        Flex::column()
            .with_child(Label::<RollShortcut>::dynamic(|shortcut, _env| format!("{}\n{}", shortcut.name, shortcut.roll)))
            .with_child(
                Flex::row()
                    .with_child(Button::new("Roll").on_click(|ctx, shortcut: &mut RollShortcut, _env| ctx.submit_command(Command::new(Selector::new("ShortcutRoll"), shortcut.clone(), Target::Global))))
                    .with_child(Button::new("Delete").on_click(|ctx, shortcut: &mut RollShortcut, _env| ctx.submit_command(Command::new(Selector::new("ShortcutDelete"), shortcut.clone(), Target::Global)))),
            )
    })
    .lens(DiceCalculator::shortcuts)
}

fn build_shortcuts_column() -> impl Widget<DiceCalculator> {
    Flex::column().with_flex_child(build_shortcut_creation_interface(), 1.).with_flex_child(Scroll::new(build_shortcut_list()), 1.)
}

fn build_main_window() -> impl Widget<DiceCalculator> {
    Flex::row()
        .with_flex_child(Align::left(build_shortcuts_column()), 1.)
        .with_child(Align::centered(build_main_column()))
        .with_flex_child(Align::right(build_history_column()), 1.)
        .controller(DiceCalcEventHandler {})
}

fn build_menu<T: Data>() -> MenuDesc<T> {
    let placeholder_command = Command::new(Selector::new("Placeholder"), (), Target::Global);
    let placeholder_entry = MenuItem::new(LocalizedString::new("Placeholder"), placeholder_command);
    let exit_command = Command::new(Selector::new("Exit"), (), Target::Global);
    let exit_entry = MenuItem::new(LocalizedString::new("Exit"), exit_command);

    MenuDesc::empty().append(placeholder_entry).append(exit_entry)
}

fn main() {
    let window = WindowDesc::new(build_main_window).title("Fluorite").menu(build_menu());
    AppLauncher::with_window(window).launch(DiceCalculator::new()).unwrap();
}
