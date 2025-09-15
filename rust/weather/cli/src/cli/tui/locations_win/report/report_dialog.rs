//! The report dialog window.
//!
use super::{criteria_dialog::CriteriaDialog, report_window::ReportWindow};
use crate::cli::reports::report_history::text;
use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect, Size},
};
use std::{ops::ControlFlow, rc::Rc};
use termui_lib::prelude::{
    beep, break_event, log_key_pressed, log_render, Button, ButtonBar, ButtonDialog, DialogResult, DialogWindow,
    MessageStyle, ReportView,
};
use weather_lib::{
    location_filter,
    prelude::{Location, WeatherData},
};

/// The criteria button identifier.
///
const CRITERIA_ID: &'static str = "CRITERIA";

/// The exit button identifier.
///
const EXIT_ID: &'static str = "EXIT";

/// The dialog that shows a locations history report.
///
pub struct ReportDialog {
    /// The report history dialog and window.
    dialog: ButtonDialog<ReportWindow>,
    /// The dialog that allows selection of history dates and category selection.
    criteria: CriteriaDialog,
    /// The name of the location.
    location_name: String,
    /// the location alias name.
    location_alias: String,
    /// The weather data history API that will be used.
    weather_data: Rc<WeatherData>,
}
impl std::fmt::Debug for ReportDialog {
    /// Show all the attributes except the weather data API.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReportDialog")
            .field("dialog", &self.dialog)
            .field("criteria", &self.criteria)
            .field("location_name", &self.location_name)
            .field("location_alias", &self.location_alias)
            .finish()
    }
}
impl ReportDialog {
    /// Create a new instance of the report dialog.
    ///
    /// # Arguments
    ///
    /// - `location` allows the name and alias to be mined.
    /// - `weather_data` is the weather history API that will be used.
    ///
    pub fn new(location: &Location, weather_data: Rc<WeatherData>) -> Self {
        Self {
            dialog: ButtonDialog::new(
                ButtonBar::new(vec![
                    Button::new(CRITERIA_ID, "Criteria", 'C').with_active(),
                    Button::new(EXIT_ID, "Exit", 'x'),
                ])
                .with_auto_select(true),
                ReportWindow::default(),
            )
            .with_title(format!(" {} Weather History", location.name)),
            criteria: CriteriaDialog::new(),
            location_name: location.name.clone(),
            location_alias: location.alias.clone(),
            weather_data,
        }
    }

    /// Get the size of the report dialog.
    ///
    pub fn size(&self) -> Size {
        self.dialog.win().size()
    }

    /// Force the dialog to recreate the history report view.
    ///
    fn refresh(&mut self) {
        match self.criteria.try_as_date_range() {
            Err(error_message) => {
                self.dialog.set_message(MessageStyle::Error, error_message);
            }
            Ok(date_range) => {
                self.criteria.set_active(false);
                let filter = location_filter!(name = &self.location_name);
                match self.weather_data.get_daily_history(filter, date_range) {
                    Err(error_message) => {
                        let message = format!("Failed to get daily history ({}).", error_message);
                        log::error!("{}", message);
                        self.dialog.set_message(MessageStyle::Error, message);
                    }
                    Ok(daily_histories) => match self.criteria.try_as_controller() {
                        Err(error_message) => {
                            self.dialog.set_message(MessageStyle::Error, error_message);
                        }
                        Ok(controller) => {
                            let report = text::Report::new(controller).with_date_format("%m/%d/%Y");
                            self.dialog.win_mut().set_view(
                                ReportView::new(report.generate(daily_histories), None)
                                    .with_show_selected(true)
                                    .with_column_labels(true)
                                    .with_horizontal_scroll(true),
                            );
                        }
                    },
                }
            }
        }
    }

    /// Dispatch a key pressed event to the report view dialog. [ControlFlow::Continue] will be
    /// returned if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    pub fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("ReportDialog");
        match self.criteria.is_active() {
            true => {
                match self.criteria.key_pressed(key_event) {
                    ControlFlow::Break(DialogResult::Continue) => (),
                    ControlFlow::Break(DialogResult::Cancel) => match self.dialog.win().is_available() {
                        true => self.criteria.set_active(false),
                        false => break_event!(DialogResult::Exit)?,
                    },
                    ControlFlow::Break(DialogResult::Exit) => {
                        self.criteria.set_active(false);
                        self.refresh();
                    }
                    ControlFlow::Continue(_) => beep(),
                    unknown => {
                        debug_assert!(false, "missed criteria result {:?}", unknown);
                        log::error!("Yikes... missed criteria result {:?}", unknown)
                    }
                }
                break_event!(DialogResult::Continue)?;
            }
            false => {
                if let ControlFlow::Break(dialog_result) = self.dialog.key_pressed(key_event) {
                    match dialog_result {
                        DialogResult::Cancel => break_event!(DialogResult::Exit)?,
                        DialogResult::Selected(id) => match id.as_str() {
                            EXIT_ID => break_event!(DialogResult::Exit)?,
                            CRITERIA_ID => {
                                // self.dialog.win_mut().set_active(true);
                                self.criteria.set_active(true);
                                break_event!(DialogResult::Continue)?;
                            }
                            _ => unreachable!(),
                        },
                        result => {
                            if DialogResult::Continue != result {
                                debug_assert!(false, "missed dialog result {:?}", result);
                                log::error!("key_pressed missed {:#?}", result);
                            }
                            break_event!(result)?
                        }
                    }
                }
            }
        }
        ControlFlow::Continue(())
    }

    /// Draw the report view dialog on the terminal screen, optionally returning the current
    /// cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the window will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    pub fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("ReportDialog");
        debug_assert!(
            self.dialog.win().is_available() || self.criteria.is_active(),
            "ReportDialog bad state, neither window or criteria active\n{:#?}",
            self
        );
        let mut coord = None;
        // if you don't have a view, don't render the dialog
        if self.dialog.win().is_available() {
            if let Some(dialog_coord) = self.dialog.render(area, buffer) {
                coord.replace(dialog_coord);
            }
        }
        if self.criteria.is_active() {
            if let Some(criteria_coord) = self.criteria.render(area, buffer) {
                coord.replace(criteria_coord);
            }
        }
        coord
    }
}
