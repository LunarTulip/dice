#![windows_subsystem = "windows"]

use druid::keyboard_types::Key;
use druid::text::format::{Formatter, Validation, ValidationError};
use druid::text::selection::Selection;
use druid::widget::prelude::*;
use druid::widget::{Align, Controller, Flex, Label, TextBox, ValueTextBox};
use druid::{AppLauncher, Command, Data, Lens, LocalizedString, MenuDesc, MenuItem, Selector, Target, Widget, WidgetExt, WindowDesc};
use fluorite::format_string_with_results;
use fluorite::parse::{clean_input, parse_input, RollInformation, VALID_INPUT_CHARS};
use std::sync::Arc;
use std::error::Error;

struct RollShortcut {
    name: String,
    roll: String,
}

#[derive(Clone, Data, Lens)]
struct DiceCalculator {
    pub current_input: String,
    pub stored_input: String,
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
}

struct KeyboardListener {}

impl<W: Widget<DiceCalculator>> Controller<DiceCalculator, W> for KeyboardListener {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut DiceCalculator, env: &Env) {
        match event {
            Event::WindowConnected => ctx.request_focus(),
            Event::MouseDown(_) => ctx.request_focus(),
            Event::KeyDown(key_event) => if ctx.is_focused() {
                match &key_event.key {
                    Key::Character(s) => {
                        if VALID_INPUT_CHARS.contains(s)  {
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
                }
            },
            _ => ()
        }
        child.event(ctx, event, data, env);
    }
}

#[derive(Debug)]
struct FormatValidationError{}

impl std::fmt::Display for FormatValidationError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Error for FormatValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

struct DiceTextFormatter {}

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

fn build_current_input_display() -> impl Widget<DiceCalculator> {
    Label::<DiceCalculator>::dynamic(|calc, _| if calc.current_input.is_empty() { String::from("Roll text") } else { String::from(&calc.current_input) })
}

fn build_main_calculator_display() -> impl Widget<DiceCalculator> {
    Label::new("Main Calculator Display Placeholder")
}

fn build_main_column() -> impl Widget<DiceCalculator> {
    Flex::column().with_flex_child(build_current_input_display(), 1.).with_flex_child(build_main_calculator_display(), 1.)
}

fn build_latest_output_display() -> impl Widget<DiceCalculator> {
    Label::<DiceCalculator>::dynamic(|calc, _| match calc.history.last() {
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
    Flex::column().with_flex_child(build_latest_output_display(), 1.).with_flex_child(build_history_display(), 1.)
}

fn build_shortcut_creation_interface() -> impl Widget<DiceCalculator> {
    Flex::column()
        .with_flex_child(TextBox::new()
            .with_placeholder("Name")
            .lens(DiceCalculator::new_shortcut_name),
            1.)
        .with_flex_child(ValueTextBox::new(
            TextBox::new()
                .with_placeholder("Roll Text"),
            DiceTextFormatter {}
        )
            .lens(DiceCalculator::new_shortcut_text),
            1.)
        .with_flex_child(Label::new("Button Placeholder"), 1.)
}

fn build_shortcuts_column() -> impl Widget<DiceCalculator> {
    Flex::column()
        .with_flex_child(build_shortcut_creation_interface(), 1.)
        .with_flex_child(Label::new("Saved Roll Shortcuts Placeholder"), 1.)
}

fn build_main_window() -> impl Widget<DiceCalculator> {
    Flex::row()
        .with_flex_child(Align::left(build_shortcuts_column()), 1.)
        .with_flex_child(Align::centered(build_main_column()), 1.)
        .with_flex_child(Align::right(build_history_column()), 1.)
        .controller(KeyboardListener {})
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
