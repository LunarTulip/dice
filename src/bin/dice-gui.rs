#![windows_subsystem = "windows"]

use dice::parse::{parse_input, RollInformation};
use druid::widget::{Align, Flex, Label, TextBox};
use druid::{AppLauncher, Command, Data, Lens, LocalizedString, MenuDesc, MenuItem, Selector, Target, Widget, WidgetExt, WindowDesc};
use std::sync::Arc;

struct RollShortcut {
    name: String,
    roll: String,
}

#[derive(Clone, Data, Lens)]
struct DiceCalculator {
    current_input: String,
    history: Arc<Vec<Result<RollInformation, String>>>,
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
        Arc::make_mut(&mut self.history).push(parse_input(&self.current_input));
        self.current_input = String::new();
    }
}

fn build_current_input_display() -> impl Widget<DiceCalculator> {
    TextBox::new()
        .with_placeholder("Roll text")
        .lens(DiceCalculator::current_input)
}

fn build_main_calculator_display() -> impl Widget<DiceCalculator> {
    Label::new("Main Calculator Display")
}

fn build_main_column() -> impl Widget<DiceCalculator> {
    Flex::column()
        .with_flex_child(build_current_input_display(), 1.)
        .with_flex_child(build_main_calculator_display(), 1.)
}

fn build_history_column() -> impl Widget<DiceCalculator> {
    Flex::column()
        .with_flex_child(Label::new("Latest Output"), 1.)
        .with_flex_child(Label::new("History"), 1.)
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
        .title("Dice Roller Which Needs A Better Name")
        .menu(build_menu());
    let calculator = DiceCalculator::new();
    AppLauncher::with_window(window)
        .launch(calculator)
        .unwrap();
}
