//! The report category dialog.

use super::criteria_window::CriteriaWindow;
use crate::cli::reports::report_history::ReportSelector;
use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
};
use std::ops::ControlFlow;
use termui_lib::prelude::{
    beep, break_event, log_key_pressed, log_render, ok_button, ButtonBar, ButtonDialog, DialogResult, DialogWindow,
    MessageStyle,
};
use weather_lib::prelude::DateRange;

#[derive(Debug)]
pub struct CriteriaDialog(ButtonDialog<CriteriaWindow>);
impl CriteriaDialog {
    /// Create a new instance of the report criteria dialog.
    ///
    pub fn new() -> Self {
        Self(
            ButtonDialog::new(
                ButtonBar::new(vec![ok_button().with_active()]).with_auto_select(true),
                CriteriaWindow::new(),
            )
            .with_title(" History Report Criteria "),
        )
    }

    /// Dispatch a key pressed event to the report criteria dialog.
    /// [ControlFlow::Continue] will be returned if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    pub fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("CriteriaDialog");
        match self.0.key_pressed(key_event) {
            ControlFlow::Break(DialogResult::Selected(_)) => match self.0.win_mut().try_as_date_range() {
                Err(error_message) => {
                    self.0.set_message(MessageStyle::Error, error_message);
                    break_event!(DialogResult::Continue)
                }
                Ok(_) => match self.0.win().try_as_report_selector() {
                    Ok(_) => {
                        self.set_active(false);
                        break_event!(DialogResult::Exit)
                    }
                    Err(error_message) => {
                        self.0.set_message(MessageStyle::Error, error_message);
                        break_event!(DialogResult::Continue)
                    }
                },
            },
            ControlFlow::Break(DialogResult::Cancel) => {
                self.set_active(false);
                break_event!(DialogResult::Cancel)
            }
            ControlFlow::Continue(()) => {
                beep();
                break_event!(DialogResult::Continue)
            }
            result => result,
        }
    }

    /// Draw the report criteria dialog on the terminal screen, optionally returning the current
    /// cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the window will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    pub fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("CriteriaDialog");
        self.0.render(area, buffer)
    }
    /// Query if the report criteria dialog is active or not.
    ///
    pub fn is_active(&self) -> bool {
        self.0.win().is_active()
    }
    /// Control if the report criteria dialog is active or not.
    ///
    /// # Arguments
    ///
    /// - `yes_no` determines if the dialog is active or not.
    ///
    pub fn set_active(&mut self, yes_no: bool) {
        self.0.win_mut().set_active(yes_no)
    }
    /// Try to get the report [date range](DateRange) from the dialog.
    ///
    pub fn try_as_date_range(&mut self) -> Result<DateRange, String> {
        self.0.win_mut().try_as_date_range()
    }
    /// Try to get the report [content selection](ReportSelector) from the dialog.
    ///
    pub fn try_as_controller(&mut self) -> Result<ReportSelector, String> {
        self.0.win_mut().try_as_report_selector()
    }
}
