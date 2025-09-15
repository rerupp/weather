//! The dialog that adds history to a location.

use crate::cli::{self, tui::validate_date};
use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect, Size},
};
use std::{cell::RefCell, ops::ControlFlow, rc::Rc};
use termui_lib::prelude::{
    beep, break_event, log_key_pressed, log_render, ok_button, ActiveNormalStyles, ButtonBar, ButtonDialog,
    ControlGroup, ControlResult, DateEditor, DialogResult, DialogWindow, EditControl, EditField, EditFieldGroup, Label,
    MessageStyle, ProgressDialog,
};
use weather_lib::prelude::{DateRange, HistoryClient, Location, WeatherData};

/// The dialog that manages adding weather data history to a location.
///
pub struct AddHistory {
    /// The weather data location.
    location: Location,
    /// A dialog that gets what history dates should be added.
    history_criteria: RefCell<ButtonDialog<HistoryCriteria>>,
    /// The dialog that actually fetches and adds history.
    history_progress: RefCell<Option<ProgressDialog>>,
    /// The history client used to download weather data.
    history_client: Box<dyn HistoryClient>,
    /// The weather data history API that will be used.
    weather_data: Rc<WeatherData>,
}
impl std::fmt::Debug for AddHistory {
    /// Show all the attributes except the weather data API.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AddHistory")
            .field("Location", &self.location)
            .field("history_criteria", &self.history_criteria)
            .field("download_progress", &self.history_progress)
            .finish()
    }
}
impl AddHistory {
    /// Create a new instance of the tab window.
    ///
    /// # Arguments
    ///
    /// - `location` allows the name and alias to be mined.
    /// - `weather_data` is the weather history API that will be used.
    ///
    pub fn new(location: &Location, weather_data: Rc<WeatherData>) -> cli::Result<Self> {
        let buttons = ButtonBar::new(vec![ok_button().with_active()]).with_auto_select(true);
        Ok(Self {
            location: location.clone(),
            history_criteria: RefCell::new(ButtonDialog::new(buttons, HistoryCriteria::new())),
            history_progress: RefCell::default(),
            history_client: weather_data.get_history_client()?,
            weather_data,
        })
    }
    /// Dispatch a key pressed event to the dialog. [ControlFlow::Continue] will be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    pub fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        let history_progress = &mut *self.history_progress.borrow_mut();
        match history_progress {
            Some(progress) => progress.key_pressed(key_event)?,
            None => {
                log_key_pressed!("AddHistory");
                let history_criteria = &mut *self.history_criteria.borrow_mut();
                if let ControlFlow::Break(result) = history_criteria.key_pressed(key_event) {
                    match result {
                        DialogResult::Exit => break_event!(DialogResult::Exit)?,
                        DialogResult::Cancel => break_event!(DialogResult::Cancel)?,
                        DialogResult::Continue => {
                            // the dialog will remain active until complete, if there is a
                            // message that was dismissed the frame will break with a Continue
                            // and this catches that condition
                            if !history_criteria.win().is_active() {
                                break_event!(DialogResult::Poll(None))?;
                            }
                        }
                        DialogResult::Selected(_) => {
                            debug_assert!(history_criteria.win().is_active(), "window bad state\n{:#?}", self);
                            match history_criteria.win_mut().try_as_date_range() {
                                Err(parse_error) => {
                                    log::error!("{parse_error}");
                                    history_criteria.set_message(MessageStyle::Error, parse_error);
                                }
                                Ok(date_range) => {
                                    let description = format!("Downloading weather history for {}", self.location.name);
                                    history_progress.replace(ProgressDialog::new(description));
                                    if let Err(error) = self.history_client.execute(&self.location, &date_range) {
                                        // this will only happen if the add history state is messed up
                                        debug_assert!(false, "{}\n{:?}", error, self)
                                    }
                                    break_event!(DialogResult::Poll(Some(20)))?;
                                }
                            }
                        }
                        DialogResult::Error(msg) => history_criteria.set_message(MessageStyle::Error, msg),
                        result => log::debug!("Yikes... view result {:?}", result),
                    }
                    break_event!(DialogResult::Continue)?;
                }
            }
        }
        ControlFlow::Continue(())
    }
    /// Draw the dialog on the terminal screen and optionally return the current cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the window will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    pub fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("AddHistory");
        let mut position = self.history_criteria.borrow().render(area, buffer);
        if self.history_progress.borrow().is_some() {
            // remove the current position since the progress dialog is running
            position.take();
            match self.history_client.poll().unwrap() {
                false => {
                    self.history_progress.borrow().as_ref().unwrap().render(area, buffer);
                }
                true => {
                    self.history_progress.take();
                    let dialog = &mut *self.history_criteria.borrow_mut();
                    // mark the window as complete and set the message about what happened
                    dialog.win_mut().set_active(false);
                    match self.history_client.get() {
                        Err(error) => dialog.set_message(MessageStyle::Error, error),
                        Ok(daily_histories) => {
                            let download_count = daily_histories.histories.len();
                            match self.weather_data.add_histories(daily_histories) {
                                Err(error) => dialog.set_message(MessageStyle::Error, error),
                                Ok(add_count) => {
                                    dialog.set_message(
                                        MessageStyle::Normal,
                                        format!("Histories downloaded {}, added {}.", download_count, add_count),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        position
    }
}

/// The field identifier for the 'from' date.
///
const FROM_ID: &'static str = "FROM";

/// The field identifier for the through date.
///
const THRU_ID: &'static str = "THRU";

/// The window that manages getting the history date range.
#[derive(Debug)]
struct HistoryCriteria {
    /// Indicates the dialog window is active or not.
    active: bool,
    /// The history date range edit fields.
    date_fields: EditFieldGroup,
}
impl HistoryCriteria {
    /// Create a new instance of the window.
    ///
    fn new() -> Self {
        let date_str = "MM/DD/YYYY";
        let date_fields = EditFieldGroup::new(vec![
            EditField::new(
                Label::align_right("From: ").with_id(FROM_ID).with_selector('F').with_active(),
                DateEditor::default(),
            ),
            EditField::new(Label::align_right("Through: ").with_id(THRU_ID).with_selector('T'), DateEditor::default()),
        ])
        .with_active()
        .with_labels_aligned()
        .with_wrap()
        .with_title(format!("History Dates ({})", date_str));
        Self { date_fields, active: true }
    }
    /// Get the history date range or return an error if there are problems.
    ///
    fn try_as_date_range(&mut self) -> Result<DateRange, String> {
        match validate_date("From", self.date_fields.get_mut(FROM_ID).unwrap().text()) {
            Err(parse_error) => {
                let _ = self.date_fields.set_active(FROM_ID);
                Err(parse_error)
            }
            Ok(from) => match validate_date("Through", self.date_fields.get(THRU_ID).unwrap().text()) {
                Err(parse_error) => {
                    let _ = self.date_fields.set_active(THRU_ID);
                    Err(parse_error)
                }
                Ok(thru) => match from <= thru {
                    false => Err(format!("Through date {} cannot be before from date {}", from, thru)),
                    true => {
                        self.set_active(false);
                        Ok(DateRange::new(from, thru))
                    }
                },
            },
        }
    }
}
impl DialogWindow for HistoryCriteria {
    /// Query if the date range window is active or not.
    ///
    fn is_active(&self) -> bool {
        self.active
    }
    /// Control if the date range window is active or not.
    ///
    /// # Arguments
    ///
    /// - `yes_no` determines if the dialog is active or not.
    ///
    fn set_active(&mut self, yes_no: bool) {
        self.active = yes_no;
    }
    /// Get the size of the window.
    ///
    fn size(&self) -> Size {
        self.date_fields.size()
    }
    /// Dispatch a key pressed event to the window. [ControlFlow::Continue] will be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("HistoryCriteria");
        if let ControlFlow::Break(control_result) = self.date_fields.key_pressed(key_event) {
            match control_result {
                ControlResult::NotAllowed | ControlResult::NextGroup | ControlResult::PrevGroup => beep(),
                _ => (),
            }
            break_event!(DialogResult::Continue)?;
        }
        ControlFlow::Continue(())
    }
    /// Draw the window on the terminal screen and optionally return the current cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the window will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("HistoryCriteria");
        self.date_fields.render(area, buffer, ActiveNormalStyles::new(self.date_fields.catalog_type))
    }
}
