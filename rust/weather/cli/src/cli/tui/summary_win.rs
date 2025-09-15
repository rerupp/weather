//! The summary tab window.

use crate::cli::{self, reports::list_summary as reports};
use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect, Size},
};
use std::{ops::ControlFlow, rc::Rc};
use termui_lib::prelude::*;
use weather_lib::{location_filters, prelude::WeatherData};

/// The main tab window showing a summary of the locations history data.
///
pub struct SummaryWindow {
    /// Indicates the tab window is active or not.
    active: bool,
    /// The location history summary report view.
    report: Option<ReportView>,
    /// The weather data history API that will be used.
    weather_data: Rc<WeatherData>,
}
impl std::fmt::Debug for SummaryWindow {
    /// Show all the attributes except the weather data API.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SummaryWindow").field("active", &self.active).field("report_view", &self.report).finish()
    }
}
impl SummaryWindow {
    /// Create a new instance of the tab window.
    ///
    /// # Arguments
    ///
    /// - `weather_data` is the weather history API that will be used.
    ///
    pub fn new(weather_data: Rc<WeatherData>) -> cli::Result<Self> {
        let mut fles = Self { active: false, report: None, weather_data };
        fles.refresh()?;
        Ok(fles)
    }
}
impl DialogWindow for SummaryWindow {
    /// Query if the tab window is active or not.
    ///
    fn is_active(&self) -> bool {
        self.active
    }

    /// Control if the tab window is active or not.
    ///
    /// # Arguments
    ///
    /// - `yes_no` determines if the dialog is active or not.
    ///
    fn set_active(&mut self, yes_no: bool) {
        self.active = yes_no;
    }

    /// Force the tab to recreate the location summary report view.
    ///
    fn refresh(&mut self) -> Result<(), String> {
        self.report.take();
        match self.weather_data.get_history_summary(location_filters!()) {
            Ok(history_summaries) => {
                let report = reports::text::Report::default().generate(history_summaries);
                self.report.replace(ReportView::new(report, None).with_show_selected(true));
                Ok(())
            }
            Err(err) => Err(format!("Summary error ({})", err)),
        }
    }

    /// Get the size of the tab window.
    ///
    fn size(&self) -> Size {
        self.report.as_ref().map_or(Size::default(), |report| report.size())
    }
    /// Dispatch a key pressed event to the tab window. [ControlFlow::Continue] will be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("SummaryWindow");
        if let Some(mut report) = self.report.take() {
            // give the report a chance to eat the event
            let result = report.key_pressed(&key_event);
            self.report.replace(report);
            if let ControlFlow::Break(control_result) = result {
                if ControlResult::NotAllowed == control_result {
                    beep();
                }
                break_event!(DialogResult::Continue)?;
            }
        } else {
            debug_assert!(false, "key_pressed bad state\n{:#?}", self);
        }
        ControlFlow::Continue(())
    }

    /// Draw the tab window on the terminal screen and optionally return the current cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the window will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("SummaryWindow");
        self.report.as_ref().map_or(None, |report| {
            let styles = report.catalog_type.get_styles(ControlState::Active);
            report.render(area, buffer, styles)
        })
    }
}
