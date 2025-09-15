//! The select report categories dialog.
use std::mem::discriminant;

use crate::cli::{reports::report_history::ReportSelector, tui::validate_date};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
};
use ratatui::{layout::Size, prelude::Alignment};
use std::{cmp, ops::ControlFlow};
use termui_lib::prelude::*;
use weather_lib::prelude::DateRange;

// use super::*;

/// The start date identifier.
///
const START_ID: &'static str = "START";

/// The end date identifier.
///
const END_ID: &'static str = "END";

/// The temperature report content identifier.
///
const TEMPERATURE_ID: &'static str = "TEMP";

/// The precipitation report content identifier.
///
const PRECIPITATION_ID: &'static str = "PRECIP";

/// The conditions report content identifier.
///
const CONDITIONS_ID: &'static str = "COND";

/// The summary report content identifier.
///
const SUMMARY_ID: &'static str = "SUM";

/// The report criteria window.
#[derive(Debug)]
pub struct CriteriaWindow {
    /// Indicates the window is active or not.
    active: bool,
    /// The report start and end dates.
    dates: EditFieldGroup,
    /// The report content selection.
    criteria: CheckBoxGroup,
    /// The size of the window.
    size: Size,
}
impl CriteriaWindow {
    /// Create a new instance of the criteria window.
    ///
    pub fn new() -> Self {
        let date_str = "MM/DD/YYYY";
        let dates = EditFieldGroup::new(vec![
            EditField::new(
                Label::align_right("Starting: ").with_id(START_ID).with_selector('S').with_active(),
                DateEditor::default(),
            ),
            EditField::new(Label::align_right("Ending: ").with_id(END_ID).with_selector('E'), DateEditor::default()),
        ])
        .with_labels_aligned()
        .with_centered_fields()
        .with_title(format!("Report Dates ({})", date_str))
        .with_title_alignment(Alignment::Center)
        .with_active();
        let criteria = CheckBoxGroup::new(vec![
            Checkbox::new(TEMPERATURE_ID, "Temperatures", 'T'),
            Checkbox::new(PRECIPITATION_ID, "Precipitation", 'P'),
            Checkbox::new(CONDITIONS_ID, "Conditions", 'n'),
            Checkbox::new(SUMMARY_ID, "Summary", 'u'),
        ])
        .with_labels_aligned()
        .with_centered_fields()
        .with_wrap()
        .with_title("Report Categories")
        .with_title_alignment(Alignment::Center);
        let dates_size = dates.size();
        let criteria_size = criteria.size();
        let size = Size {
            width: cmp::max(dates_size.width, criteria_size.width),
            height: dates_size.height + criteria_size.height + 1,
        };
        Self { active: true, dates, criteria, size }
    }

    /// Try to get the report [date range](DateRange) from the window.
    ///
    pub fn try_as_date_range(&mut self) -> Result<DateRange, String> {
        match validate_date("From", self.dates.get_mut(START_ID).unwrap().text()) {
            Err(parse_error) => {
                let _ = self.dates.set_active(START_ID);
                Err(parse_error)
            }
            Ok(start) => match validate_date("Through", self.dates.get(END_ID).unwrap().text()) {
                Err(parse_error) => {
                    let _ = self.dates.set_active(END_ID);
                    Err(parse_error)
                }
                Ok(end) => match start <= end {
                    false => Err(format!("Start date {} cannot be before end date {}", start, end)),
                    true => Ok(DateRange::new(start, end)),
                },
            },
        }
    }

    /// Try to get the report [content selection](ReportSelector) from the window.
    ///
    pub fn try_as_report_selector(&self) -> Result<ReportSelector, String> {
        let temperatures = self.criteria.get(TEMPERATURE_ID).unwrap().is_checked();
        let precipitation = self.criteria.get(PRECIPITATION_ID).unwrap().is_checked();
        let conditions = self.criteria.get(CONDITIONS_ID).unwrap().is_checked();
        let summary = self.criteria.get(SUMMARY_ID).unwrap().is_checked();
        match temperatures || precipitation || conditions || summary {
            true => Ok(ReportSelector { temperatures, precipitation, conditions, summary }),
            false => Err("A report category must be selected.".to_string()),
        }
    }
}
impl DialogWindow for CriteriaWindow {
    /// Query if the report criteria window is active or not.
    ///
    fn is_active(&self) -> bool {
        self.active
    }

    /// Control if the report criteria window is active or not.
    ///
    /// # Arguments
    ///
    /// - `yes_no` determines if the dialog is active or not.
    ///
    fn set_active(&mut self, yes_no: bool) {
        self.active = yes_no;
    }

    /// Get the size of the report criteria window.
    ///
    fn size(&self) -> Size {
        self.size
    }

    /// Dispatch a key pressed event to the report criteria window. [ControlFlow::Continue] will be
    /// returned if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("CriteriaWindow");
        macro_rules! toggle_active_group {
            () => {
                self.dates.active = !self.dates.active;
                self.criteria.active = !self.dates.active;
            };
        }
        // check the event to see if it is a field selector
        let is_selector = match key_event.modifiers == KeyModifiers::ALT {
            true => discriminant(&key_event.code) == discriminant(&KeyCode::Char(' ')),
            false => false,
        };
        let control_result = match self.dates.active {
            true => match self.dates.key_pressed(key_event) {
                ControlFlow::Continue(_) => match is_selector {
                    false => ControlFlow::Continue(()),
                    true => {
                        // if dates didn't handle the event then try criteria
                        let criteria_result = self.criteria.key_pressed(key_event);
                        if criteria_result.is_break() {
                            toggle_active_group!();
                            self.dates.clear_active();
                        }
                        criteria_result
                    }
                },
                dates_result => dates_result,
            },
            false => match self.criteria.key_pressed(key_event) {
                ControlFlow::Continue(_) => match is_selector {
                    false => ControlFlow::Continue(()),
                    true => {
                        // if criteria didn't handle the event then try dates
                        let dates_result = self.dates.key_pressed(key_event);
                        if dates_result.is_break() {
                            toggle_active_group!();
                            self.criteria.clear_active();
                        }
                        dates_result
                    }
                },
                criteria_result => criteria_result,
            },
        };
        if let ControlFlow::Break(control_result) = control_result {
            // log::debug!("control result {:?}", control_result);
            match control_result {
                ControlResult::Continue => (),
                ControlResult::NotAllowed => beep(),
                ControlResult::Selected(id) => {
                    let id_str = id.as_str();
                    if id_str == START_ID || id_str == END_ID {
                        let _ = self.dates.set_active(id);
                    } else {
                        let _ = self.criteria.set_active(id);
                    }
                }
                ControlResult::NextGroup => {
                    if self.dates.active {
                        self.dates.clear_active();
                        self.criteria.set_first_active();
                    } else {
                        self.criteria.clear_active();
                        self.dates.set_first_active();
                    }
                    toggle_active_group!();
                }
                ControlResult::PrevGroup => {
                    if self.dates.active {
                        self.dates.clear_active();
                        self.criteria.set_last_active();
                    } else {
                        self.criteria.clear_active();
                        self.dates.set_last_active();
                    }
                    toggle_active_group!();
                }
                unknown => {
                    debug_assert!(false, "control result not handled {:?}", unknown);
                    log::error!("window result not handled {:?}", unknown);
                }
            }
            break_event!(DialogResult::Continue)?;
        }
        ControlFlow::Continue(())
    }

    /// Draw the report criteria window on the terminal screen, optionally returning the current
    /// cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the window will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        if !self.active {
            None?;
        }
        log_render!("CriteriaWindow");
        // show the date group
        let dates_height = self.dates.size().height as i32;
        let dates_area = inner_rect(area, (0, 0), (0, dates_height));
        let styles = match self.dates.active {
            true => ActiveNormalStyles::new(self.dates.catalog_type),
            false => ActiveNormalStyles::with_active_style(self.dates.catalog_type, ControlState::Normal),
        };
        let mut coord = self.dates.render(dates_area, buffer, styles);
        // show the criteria group
        let criteria_area = inner_rect(area, (0, dates_height + 1), (0, 0));
        let styles = match self.criteria.active {
            true => ActiveNormalStyles::new(self.criteria.catalog_type),
            false => ActiveNormalStyles::with_active_style(self.dates.catalog_type, ControlState::Normal),
        };
        if let Some(criteria_coord) = self.criteria.render(criteria_area, buffer, styles) {
            coord.replace(criteria_coord);
        }
        coord
    }
}
