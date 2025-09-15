//! The location UI menus.
//! 
use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
};
use std::ops::ControlFlow;
use termui_lib::prelude::{
    beep, break_event, log_key_pressed, log_render, ControlResult, DialogResult, MenuItem, PopupMenu,
};

/// The popup menu history identifier.
///
pub const ADD_ID: &'static str = "ADD";

/// The popup menu report history identifier.
///
pub const REPORT_ID: &'static str = "REPORT";

#[derive(Debug)]
pub struct ContextMenu(PopupMenu);
impl ContextMenu {
    /// Create a new instance of the context menu.
    ///
    pub fn new() -> Self {
        Self(PopupMenu::new(vec![
            MenuItem::new(ADD_ID, "Add History", 'A'),
            MenuItem::new(REPORT_ID, "History Report", 'R'),
        ]))
    }
    /// Draw the context menu on the terminal screen and optionally return the current cursor position.
    ///
    /// # Arguments
    ///
    /// - `position` is the upper left corner of where the context menu will be drawn on the screen.
    /// - `area` is where on the terminal screen the window will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    pub fn render(&self, position: Position, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("ContextMenu");
        self.0.render(position, area, buffer)
    }
    /// Get the result of a key pressed event for the popup menu.
    ///
    /// # Returns
    ///
    /// * DialogResult::Exit when the popup is complete.
    /// * DialogResult::Selected when a menu action is selected.
    /// * DialogResult::Continue if the menu is not complete.
    /// * ControlFlow::Continue if the menu did not process the key event.
    ///
    pub fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("ContextMenu");
        if let ControlFlow::Break(control_result) = self.0.key_pressed(key_event) {
            match control_result {
                ControlResult::Cancel => break_event!(DialogResult::Exit)?,
                ControlResult::Selected(id) => break_event!(DialogResult::Selected(id))?,
                ControlResult::Continue => break_event!(DialogResult::Continue)?,
                ControlResult::NotAllowed => {
                    beep();
                    break_event!(DialogResult::Continue)?
                }
                unknown => {
                    debug_assert!(false, "LocationMenu missed popup result {:?}\n{:#?}", unknown, self);
                }
            }
        }
        ControlFlow::Continue(())
    }
}
