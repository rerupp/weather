//! The TUI for inline views and dialogs.

mod command_bar;
mod editor;
mod editor_group;
mod label;

use crossterm::{cursor, event, execute};
use ratatui::{DefaultTerminal, TerminalOptions, Viewport};

pub use {
    command_bar::{CommandBar, CommandBarStyles},
    editor::{Editor, EditorResult, EditorStyles},
    editor_group::{EditorGroup, EditorGroupResult, EditorGroupStyles},
    label::{Label, LabelStyles},
};

pub fn beep() {
    use std::io::Write;
    if let Err(err) = std::io::stdout().write(&[7]) {
        log::error!("Could not beep terminal ({}).", err);
    }
}

/// Run a TUI in an inline viewport on the terminal screen.
///
/// # Arguments
///
/// * `rows` sets the size of the inline viewport.
/// * `run` is a function that runs the TUI.
///
pub fn run_viewport<F, T>(rows: u16, mut run: F) -> T
where
    F: FnMut(&mut DefaultTerminal) -> T,
{
    // create an inline viewport
    let mut terminal = ratatui::init_with_options(TerminalOptions { viewport: Viewport::Inline(rows) });

    // set up positioning the cursor when the tui is complete
    let mut cursor_position = terminal.get_cursor_position().unwrap();
    cursor_position.y += rows;

    // make sure the cursor will be shown
    execute!(terminal.backend_mut(), cursor::SetCursorStyle::DefaultUserShape, event::EnableMouseCapture).unwrap();

    // run the tui
    let editor_result = run(&mut terminal);

    // set the cursor on the line following the tui
    terminal.set_cursor_position(cursor_position).unwrap();

    // return the result
    editor_result
}
