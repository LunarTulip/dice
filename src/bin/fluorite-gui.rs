#![windows_subsystem = "windows"]

use druid::commands::QUIT_APP;
use druid::keyboard_types::Key;
use druid::text::format::{Formatter, Validation, ValidationError};
use druid::text::selection::Selection;
use druid::widget::prelude::*;
use druid::widget::{Align, Button, Controller, Flex, Label, LineBreaking, List, Scroll, SizedBox, Split, TextBox, ValueTextBox};
use druid::{AppLauncher, Command, Data, Lens, LocalizedString, MenuDesc, MenuItem, Selector, Target, Widget, WidgetExt, WindowDesc};
use fluorite::parse::{clean_input, parse_input, RollInformation, VALID_INPUT_CHARS};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::env::current_exe;
use std::error::Error;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;
use std::sync::Arc;

////////////////
//   Consts   //
////////////////

lazy_static! {
    static ref DATA_DIR: PathBuf = {
        let mut path = current_exe().unwrap(); // Replace with real error-handling
        path.pop();
        path.push("fluorite_data");

        path
    };
    static ref CONFIG_PATH: PathBuf = {
        let mut path = DATA_DIR.clone();
        path.push("config.json");

        path
    };
    static ref HISTORY_PATH: PathBuf = {
        let mut path = DATA_DIR.clone();
        path.push("history.json");

        path
    };
    static ref SHORTCUTS_PATH: PathBuf = {
        let mut path = DATA_DIR.clone();
        path.push("shortcuts.json");

        path
    };
}

/////////////////
//   Structs   //
/////////////////

#[derive(Clone, Data, Deserialize, Serialize)]
struct DiceCalculatorConfig {
    max_history_entries: u64,
    save_history: bool,
    save_shortcuts: bool,
}

impl DiceCalculatorConfig {
    fn new() -> DiceCalculatorConfig {
        DiceCalculatorConfig {
            max_history_entries: 100,
            save_history: true,
            save_shortcuts: true,
        }
    }
}

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

#[derive(Clone, Data, Deserialize, Serialize)]
struct RollShortcut {
    name: String,
    roll: String,
}

#[derive(Clone, Data, Lens)]
struct DiceCalculator {
    config: DiceCalculatorConfig,
    current_input: String,
    stored_input: String,
    history: Arc<Vec<(String, Result<RollInformation, String>)>>,
    steps_back_in_history: usize,
    shortcuts: Arc<Vec<RollShortcut>>,
    new_shortcut_name: String,
    new_shortcut_text: String,
}

impl DiceCalculator {
    fn new(config: DiceCalculatorConfig) -> DiceCalculator {
        DiceCalculator {
            config: config.clone(),
            current_input: String::new(),
            stored_input: String::new(),
            history: match config.save_history {
                true => load_history(),
                false => Arc::new(Vec::new()),
            },
            steps_back_in_history: 0,
            shortcuts: match config.save_history {
                true => load_shortcuts(),
                false => Arc::new(Vec::new()),
            },
            new_shortcut_name: String::new(),
            new_shortcut_text: String::new(),
        }
    }
    fn add_to_history(&mut self, input: String, output: Result<RollInformation, String>) {
        let history = Arc::make_mut(&mut self.history);
        history.push((input, output));
        while history.len() as u64 > self.config.max_history_entries {
            let _ = history.drain(0..1);
        }
        if self.config.save_history {
            save_history(&self);
        }
    }
    fn roll(&mut self) {
        if !self.current_input.is_empty() {
            self.add_to_history(self.current_input.clone(), parse_input(&self.current_input));
            self.current_input = String::new();
            self.stored_input = String::new();
            self.steps_back_in_history = 0;
        }
    }
    fn roll_from_shortcut(&mut self, shortcut: &RollShortcut) {
        self.add_to_history(shortcut.roll.clone(), parse_input(&shortcut.roll));
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
        }
        if data.config.save_shortcuts {
            save_shortcuts(&data);
        }
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
                    let shortcut = &command.get_unchecked::<RollShortcut>(Selector::new("ShortcutRoll"));
                    data.roll_from_shortcut(shortcut);
                } else if command.is::<RollShortcut>(Selector::new("ShortcutDelete")) {
                    let name_to_delete = command.get_unchecked::<RollShortcut>(Selector::new("ShortcutDelete")).name.clone();
                    Arc::make_mut(&mut data.shortcuts).retain(|shortcut| shortcut.name != name_to_delete);
                    if data.config.save_shortcuts {
                        save_shortcuts(&data);
                    }
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

//////////////////////////
//   Helper Functions   //
//////////////////////////

fn ensure_data_dir_exists() {
    if !DATA_DIR.as_path().exists() {
        create_dir_all(&*DATA_DIR).unwrap()
    }
}

fn load_config() -> DiceCalculatorConfig {
    match read_to_string(&*CONFIG_PATH) {
        Ok(config_as_json) => match serde_json::from_str(&config_as_json) {
            Ok(config) => config,
            Err(_) => DiceCalculatorConfig::new(),
        },
        Err(_) => DiceCalculatorConfig::new(),
    }
}

fn load_history() -> Arc<Vec<(String, Result<RollInformation, String>)>> {
    match read_to_string(&*HISTORY_PATH) {
        Ok(history_as_json) => match serde_json::from_str(&history_as_json) {
            Ok(history) => history,
            Err(_) => Arc::new(Vec::new()),
        },
        Err(_) => Arc::new(Vec::new()),
    }
}

fn load_shortcuts() -> Arc<Vec<RollShortcut>> {
    match read_to_string(&*SHORTCUTS_PATH) {
        Ok(shortcuts_as_json) => match serde_json::from_str(&shortcuts_as_json) {
            Ok(shortcuts) => shortcuts,
            Err(_) => Arc::new(Vec::new()),
        },
        Err(_) => Arc::new(Vec::new()),
    }
}

fn save_config(calc: &DiceCalculator) {
    ensure_data_dir_exists();
    let config_as_json = serde_json::to_string(&calc.config).unwrap();
    write(&*CONFIG_PATH, config_as_json).unwrap();
}

fn save_history(calc: &DiceCalculator) {
    ensure_data_dir_exists();
    let history_as_json = serde_json::to_string(&calc.history).unwrap();
    write(&*HISTORY_PATH, history_as_json).unwrap();
}

fn save_shortcuts(calc: &DiceCalculator) {
    ensure_data_dir_exists();
    let shortcuts_as_json = serde_json::to_string(&calc.shortcuts).unwrap();
    write(&*SHORTCUTS_PATH, shortcuts_as_json).unwrap();
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
            CalcButton::Roll => data.roll(), // TODO: ban empty rolls
            CalcButton::PlaceholderNotCurrentlyInUse => (),
        })
}

fn build_current_input_display() -> impl Widget<DiceCalculator> {
    Label::<DiceCalculator>::dynamic(|calc, _env| if calc.current_input.is_empty() { String::from("Roll text") } else { String::from(&calc.current_input) }).with_text_size(50.)
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
    Split::rows(Align::right(build_current_input_display()), build_main_calculator_display()).split_point(0.15).solid_bar(true)
}

fn build_latest_output_display() -> impl Widget<DiceCalculator> {
    Label::<DiceCalculator>::dynamic(|calc, _env| match calc.history.last() {
        None => String::from("Result"),
        Some(roll_result) => match &roll_result.1 {
            Err(_) => String::from("Error"),
            Ok(info) => format!("{}", info.value),
        },
    })
    .with_text_size(50.)
}

fn build_history_display() -> impl Widget<DiceCalculator> {
    Label::<DiceCalculator>::dynamic(|calc, _| {
        let mut history = String::new();
        for roll_result in calc.history.iter().rev() {
            match roll_result {
                (input, Err(e)) => history.push_str(&format!("Input: {}\nError: {}\n\n", input, e)),
                (input, Ok(info)) => history.push_str(&format!("Input: {}\nRolled: {}\nResult: {}\n\n", input, info.processed_string, info.value)),
            }
        }
        history
    })
    .with_line_break_mode(LineBreaking::WordWrap)
}

fn build_history_column() -> impl Widget<DiceCalculator> {
    Split::rows(Align::centered(build_latest_output_display()), Scroll::new(build_history_display())).split_point(0.15).solid_bar(true)
}

fn build_shortcut_creation_interface() -> impl Widget<DiceCalculator> {
    Flex::column()
        .with_child(TextBox::new().with_placeholder("Name").lens(DiceCalculator::new_shortcut_name))
        .with_child(ValueTextBox::new(TextBox::new().with_placeholder("Roll Text"), DiceTextFormatter {}).lens(DiceCalculator::new_shortcut_text))
        .with_child(Button::new("Create Shortcut").on_click(DiceCalculator::add_shortcut))
}

fn build_shortcut_list() -> impl Widget<DiceCalculator> {
    List::new(|| {
        Flex::column()
            .with_child(Label::<RollShortcut>::dynamic(|shortcut, _env| format!("{}\n{}", shortcut.name, shortcut.roll)).with_line_break_mode(LineBreaking::WordWrap))
            .with_child(
                Flex::row()
                    .with_child(Button::new("Roll").on_click(|ctx, shortcut: &mut RollShortcut, _env| ctx.submit_command(Command::new(Selector::new("ShortcutRoll"), shortcut.clone(), Target::Global))))
                    .with_child(Button::new("Delete").on_click(|ctx, shortcut: &mut RollShortcut, _env| ctx.submit_command(Command::new(Selector::new("ShortcutDelete"), shortcut.clone(), Target::Global)))),
            )
    })
    .lens(DiceCalculator::shortcuts)
}

fn build_shortcuts_column() -> impl Widget<DiceCalculator> {
    Split::rows(Align::centered(build_shortcut_creation_interface()), Scroll::new(Align::centered(build_shortcut_list())))
        .split_point(0.15)
        .solid_bar(true)
}

fn build_main_window() -> impl Widget<DiceCalculator> {
    Split::columns(
        Split::columns(build_shortcuts_column(), build_main_column()).split_point(0.33).solid_bar(true).draggable(true),
        build_history_column(),
    )
    .split_point(0.75)
    .solid_bar(true)
    .draggable(true)
    .controller(DiceCalcEventHandler {})
}

fn build_file_menu<T: Data>() -> MenuDesc<T> {
    let exit = MenuItem::new(LocalizedString::new("Exit"), QUIT_APP);

    MenuDesc::new(LocalizedString::new("File")).append(exit)
}

fn build_menus<T: Data>() -> MenuDesc<T> {
    MenuDesc::empty().append(build_file_menu())
}

fn main() {
    let config = load_config();
    let calculator = DiceCalculator::new(config);
    save_config(&calculator);

    let window = WindowDesc::new(build_main_window).title("Fluorite").menu(build_menus());

    AppLauncher::with_window(window).launch(calculator).unwrap();
}
