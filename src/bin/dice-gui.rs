use druid::widget::{Align, Flex, Label, Padding};
use druid::{AppLauncher, UnitPoint, Widget, WindowDesc};

fn build_ui() -> impl Widget<()> {
    Padding::new(
        10.,
        Align::new(
            UnitPoint::CENTER,
            Flex::row()
                .with_flex_child(
                    Flex::column()
                        .with_flex_child(Label::new("Saved Roll Shortcuts"), 1.),
                        1.
                )
                .with_flex_child(
                    Flex::column()
                        .with_flex_child(Label::new("Current Input"), 1.)
                        .with_flex_child(Label::new("Main Calculator Display"), 1.),
                        1.
                )
                .with_flex_child(
                    Flex::column()
                        .with_flex_child(Label::new("Latest Output"), 1.)
                        .with_flex_child(Label::new("History"), 1.),
                        1.
                )
        )
    )
}

fn main() {
    AppLauncher::with_window(
        WindowDesc::new(|| build_ui())
            .title("Dice Roller Which Needs A Better Name")
    ).launch(()).unwrap();
}
