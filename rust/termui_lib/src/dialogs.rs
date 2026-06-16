//! The TUI dialog windows.
//!
//! There are 2 basic categories of windows.
//!
//! - The [TabDialog] manages a collection of windows in a tabbed window.
//! - The [Modal](modal) dialogs are used to manage a viewing content and providing
//! information. Generally `modal` dialogs appear in front of the application.
//!
//! All dialogs implement the [DialogWindow] trait.
//!
use super::*;
use controls::ControlResult;
pub use modal::{ButtonDialog, MenuDialog, MessageDialog, MessageStyle, ProgressDialog};
use ratatui::layout::Position;
use styles::{CatalogType, ControlState, StyleCatalog, StyleId};
pub use tabbed::{TabDialog, TabWindow};

mod modal;
mod tabbed;

/// The result of some dialog operation.
///
#[derive(Debug, PartialOrd, PartialEq)]
pub enum DialogResult {
    /// Indicate the dialog is not complete and should continue receiving events.
    Continue,
    /// Indicate the dialog did not complete and should not receive events.
    Cancel,
    /// Indicate the dialog has completed and should not receive events.
    Exit,
    /// Indicate the window had an unrecoverable error and should terminate.
    Error(String),
    /// Allow the dialog to indicate it wants a specific poll interval.
    Poll(Option<usize>),
    /// Return the id of some selected control or dialog item. The implementation decides
    /// if it should continue or exit.
    Selected(String),
}

/// The API used by the dialog windows.
///
pub trait DialogWindow: std::fmt::Debug {
    /// Query if the window is active or not.
    ///
    fn is_active(&self) -> bool;
    /// Control if the window is active or not.
    ///
    /// # Arguments
    ///
    /// - `yes_no` determines if the window is active or not.
    ///
    fn set_active(&mut self, yes_no: bool);
    /// Instruct the window to refresh its content.
    ///
    fn refresh(&mut self) -> std::result::Result<(), String> {
        Ok(())
    }
    /// Get the size of the window.
    ///
    fn size(&self) -> Size;
    /// Dispatch a key pressed event to the window. [ControlFlow::Continue] should be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult>;
    /// Draw the window on the terminal screen and optionally return the current cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the window will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position>;
}
