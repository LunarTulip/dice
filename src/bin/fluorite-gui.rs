#![windows_subsystem = "windows"]

use druid::keyboard_types::Key;
use druid::widget::prelude::*;
use druid::widget::{Align, Controller, Flex, Label};
use druid::{AppLauncher, Command, Data, Lens, LocalizedString, MenuDesc, MenuItem, Selector, Target, Widget, WidgetExt, WindowDesc};
use fluorite::parse::{parse_input, RollInformation, VALID_INPUT_CHARS};
use fluorite::format_string_with_results;
use std::sync::Arc;

struct RollShortcut {
    name: String,
    roll: String,
}

#[derive(Clone, Data, Lens)]
struct DiceCalculator {
    pub current_input: String,
    history: Arc<Vec<(String, Result<RollInformation, String>)>>,
    shortcuts: Arc<Vec<RollShortcut>>,
}

impl DiceCalculator {
    fn new() -> DiceCalculator {
        DiceCalculator {
            current_input: String::new(),
            history: Arc::new(Vec::new()),
            shortcuts: Arc::new(Vec::new()),
        }
    }
    fn roll(&mut self) {
        Arc::make_mut(&mut self.history).push((self.current_input.clone(), parse_input(&self.current_input)));
        self.current_input = String::new();
    }
}

struct KeyboardListener {}

impl<W: Widget<DiceCalculator>> Controller<DiceCalculator, W> for KeyboardListener {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut DiceCalculator, env: &Env) {
        ctx.request_focus();
        if let Event::KeyDown(key_event) = event {
            match &key_event.key {
                Key::Character(s) => {
                    if VALID_INPUT_CHARS.contains(s) {
                        data.current_input.push_str(&s);
                    }
                }
                Key::Backspace => {
                    let _ = data.current_input.pop();
                }
                Key::Enter => data.roll(),
                _ => (),
            }
        }
        child.event(ctx, event, data, env);
    }
}

fn build_current_input_display() -> impl Widget<DiceCalculator> {
    Label::<DiceCalculator>::dynamic(|calc, _| {
        if calc.current_input.is_empty() {
            String::from("Roll text")
        } else {
            String::from(&calc.current_input)
        }
    })
}

fn build_main_calculator_display() -> impl Widget<DiceCalculator> {
    Label::new("Main Calculator Display")
}

fn build_main_column() -> impl Widget<DiceCalculator> {
    Flex::column()
        .with_flex_child(build_current_input_display(), 1.)
        .with_flex_child(build_main_calculator_display(), 1.)
}

fn build_latest_output_display() -> impl Widget<DiceCalculator> {
    Label::<DiceCalculator>::dynamic(|calc, _| {
        match calc.history.last() {
            None => String::new(),
            Some(roll_result) => match &roll_result.1 {
                Err(_) => String::from("ERROR"),
                Ok(info) => format!("{}", info.value),
            }
        }
    })
}

fn build_history_display() -> impl Widget<DiceCalculator> {
    Label::<DiceCalculator>::dynamic(|calc, _| {
        let mut history = String::new();
        for roll_result in calc.history.iter().rev() {
            match roll_result {
                (input, Err(e)) => history.push_str(&format!("Input: {}\nError: {}\n\n", input, e)),
                (input, Ok(info)) => history.push_str(&format!("Input: {}\nRolled: {}\nResult: {}\n\n", input, format_string_with_results(&info.processed_string, info.rolls.clone()), info.value)),
            }
        }
        history
    })
}

fn build_history_column() -> impl Widget<DiceCalculator> {
    Flex::column()
        .with_flex_child(build_latest_output_display(), 1.)
        .with_flex_child(build_history_display(), 1.)
}

fn build_shortcuts_column() -> impl Widget<DiceCalculator> {
    Flex::column()
        .with_flex_child(Label::new("Create-A-Shortcut Interface"), 1.)
        .with_flex_child(Label::new("Saved Roll Shortcuts"), 1.)
}

fn build_main_window() -> impl Widget<DiceCalculator> {
    Flex::row()
        .with_flex_child(
            Align::left(build_shortcuts_column()),
            1.
        )
        .with_flex_child(
            Align::centered(build_main_column()),
            1.
        )
        .with_flex_child(
            Align::right(build_history_column()),
            1.
        )
        .controller(KeyboardListener {})
}

fn build_menu<T: Data>() -> MenuDesc<T> {
    let placeholder_command = Command::new(Selector::new("Placeholder"), (), Target::Global);
    let placeholder_entry = MenuItem::new(LocalizedString::new("Placeholder"), placeholder_command);
    let exit_command = Command::new(Selector::new("Exit"), (), Target::Global);
    let exit_entry = MenuItem::new(LocalizedString::new("Exit"), exit_command);

    MenuDesc::empty()
        .append(placeholder_entry)
        .append(exit_entry)
}

fn main() {
    let window = WindowDesc::new(build_main_window)
        .title("Fluorite")
        .menu(build_menu());
    AppLauncher::with_window(window)
        .launch(DiceCalculator::new())
        .unwrap();
}
