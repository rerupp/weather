//! The report window.

use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect, Size},
};
use std::ops::ControlFlow;
use termui_lib::prelude::{
    beep, break_event, log_key_pressed, log_render, Control, ControlResult, ControlState, DialogResult, DialogWindow,
    ReportView,
};

/// The location history report window.
///
#[derive(Debug, Default)]
pub struct ReportWindow {
    /// Indicates the window is active or not.
    active: bool,
    /// The location history report view.
    view: Option<ReportView>,
}
impl ReportWindow {
    /// Get the report view that will be drawn.
    ///
    pub fn is_available(&self) -> bool {
        self.view.is_some()
    }

    /// Change the report view that will be drawn.
    ///
    /// # Arguments
    ///
    /// - `view` is the new report view.
    ///
    pub fn set_view(&mut self, view: ReportView) {
        self.view.replace(view);
    }
}
impl DialogWindow for ReportWindow {
    /// Query if the report view is active or not.
    ///
    fn is_active(&self) -> bool {
        self.active
    }

    /// Control if the report view is active or not.
    ///
    /// # Arguments
    ///
    /// - `yes_no` determines if the dialog is active or not.
    ///
    fn set_active(&mut self, yes_no: bool) {
        self.active = yes_no;
    }

    /// Get the size of the report view.
    ///
    fn size(&self) -> Size {
        match self.view.as_ref() {
            None => Size::default(),
            Some(view) => view.size(),
        }
    }

    /// Dispatch a key pressed event to the report view window. [ControlFlow::Continue] will be
    /// returned if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("ReportWindow");
        if let Some(view) = self.view.as_mut() {
            if let ControlFlow::Break(control_result) = view.key_pressed(&key_event) {
                if ControlResult::NotAllowed == control_result {
                    beep();
                }
                break_event!(DialogResult::Continue)?;
            }
        }
        ControlFlow::Continue(())
    }

    /// Draw the report view window on the terminal screen, optionally returning the current
    /// cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the window will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("ReportWindow");
        let view = self.view.as_ref()?;
        let coord = view.render(area, buffer, view.catalog_type.get_styles(ControlState::Active))?;
        Some(coord)
    }
}
