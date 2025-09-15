//! The locations information window.

use crate::cli::{self, reports::list_locations as reports};
use add_history::AddHistory;
use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect, Size},
};
use std::{ops::ControlFlow, rc::Rc};
use termui_lib::prelude::{
    beep, break_event, log_key_pressed, log_render, Control, ControlResult, ControlState, DialogResult, DialogWindow,
    ReportView,
};
use weather_lib::{
    location_filters,
    prelude::{Location, WeatherData},
};

mod add_history;
mod context_menu;
mod report;

// The metadata that associates a collection of locations with a report view.
#[derive(Debug)]
struct LocationsView {
    /// The collection of locations.
    locations: Vec<Location>,
    /// The report view of the location collection.
    view: ReportView,
}
impl LocationsView {
    /// Return the location information for the current row in the report view.
    ///
    fn selected_location(&self) -> &Location {
        let selected_row = self.view.selected_row();
        debug_assert!(selected_row < self.locations.len(), "View selected row oob. {:?}", self);
        self.locations.get(selected_row).unwrap()
    }
}

/// The main tab window showing the locations that are available.
///
pub struct LocationsWindow {
    /// Indicates the window is active or not.
    active: bool,
    /// The current location information.
    locations_view: Option<LocationsView>,
    /// The popup menu for a location
    popup: Option<context_menu::ContextMenu>,
    /// The add histories dialog.
    add_history: Option<AddHistory>,
    /// Placeholder for the report dialog.
    report_win: Option<report::ReportDialog>,
    /// The weather data API.
    weather_data: Rc<WeatherData>,
}
impl std::fmt::Debug for LocationsWindow {
    /// Show all the attributes except the weather data API.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocationsWindow")
            .field("active", &self.active)
            .field("locations_view", &self.locations_view)
            .field("popup", &self.popup)
            .field("add_history", &self.add_history)
            .field("report_win", &self.report_win)
            .finish()
    }
}
impl LocationsWindow {
    /// Create a new instance of the tab window.
    ///
    /// # Arguments
    ///
    /// - `weather_data` is the weather history API that will be used.
    ///
    pub fn new(weather_data: Rc<WeatherData>) -> cli::Result<Self> {
        let mut me = Self {
            active: false,
            locations_view: None,
            popup: None,
            add_history: None,
            report_win: None,
            weather_data,
        };
        me.refresh()?;
        Ok(me)
    }

    /// Dispatch a key pressed event to the popup menu. [ControlFlow::Continue] will be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    fn popup_key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        debug_assert!(self.popup.is_some(), "popup is missing.\n{:#?}", self);
        let mut popup = self.popup.take().unwrap();
        match popup.key_pressed(key_event) {
            ControlFlow::Continue(()) => {
                self.popup.replace(popup);
            }
            ControlFlow::Break(dialog_result) => match dialog_result {
                DialogResult::Selected(id) => match id.as_str() {
                    context_menu::ADD_ID => {
                        let location = self.locations_view.as_ref().unwrap().selected_location();
                        match AddHistory::new(location, self.weather_data.clone()) {
                            Err(error) => break_event!(DialogResult::Error(error.to_string()))?,
                            Ok(add_history) => {
                                self.add_history.replace(add_history);
                                break_event!(DialogResult::Continue)?;
                            }
                        }
                    }
                    context_menu::REPORT_ID => {
                        let location = self.locations_view.as_ref().unwrap().selected_location();
                        self.report_win.replace(report::ReportDialog::new(location, self.weather_data.clone()));
                        break_event!(DialogResult::Continue)?;
                    }
                    _ => unreachable!(),
                },
                DialogResult::Exit => break_event!(DialogResult::Continue)?,
                DialogResult::Continue => {
                    self.popup.replace(popup);
                    break_event!(DialogResult::Continue)?;
                }
                unknown => {
                    debug_assert!(false, "popup_key_pressed break missed {:#?}\n{:#?}", unknown, self)
                }
            },
        }
        ControlFlow::Continue(())
    }

    /// Dispatch a key pressed event to the add history dialog. [ControlFlow::Continue] will be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    fn add_history_key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        debug_assert!(self.add_history.is_some(), "LocationsWindow AddHistory is None\n{:#?}", self);
        let mut dialog = self.add_history.take().unwrap();
        match dialog.key_pressed(key_event) {
            ControlFlow::Break(DialogResult::Cancel) => {
                break_event!(DialogResult::Poll(None))
            }
            ControlFlow::Break(DialogResult::Exit) => {
                // todo: you need to send back a refresh so the windows will be updated
                break_event!(DialogResult::Poll(None))
            }
            ControlFlow::Break(DialogResult::Poll(timeout)) => {
                if timeout.is_some() {
                    // since the dialog is setting the poll interval it's still active
                    self.add_history.replace(dialog);
                }
                break_event!(DialogResult::Poll(timeout))
            }
            ControlFlow::Break(result) => {
                // add history is still active
                self.add_history.replace(dialog);
                break_event!(result)
            }
            ControlFlow::Continue(()) => {
                // since the dialog didn't consume the key it's still active
                self.add_history.replace(dialog);
                ControlFlow::Continue(())
            }
        }
    }

    /// Dispatch a key pressed event to the tab window report view. [ControlFlow::Continue] will be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    fn history_view_key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        debug_assert!(self.report_win.is_some(), "report window missing\n{:#?}", self);
        if let Some(mut dialog) = self.report_win.take() {
            if ControlFlow::Break(DialogResult::Exit) != dialog.key_pressed(key_event) {
                self.report_win.replace(dialog);
            }
            break_event!(DialogResult::Continue)?;
        }
        ControlFlow::Continue(())
    }
}
impl DialogWindow for LocationsWindow {
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

    /// Force the tab to recreate the locations view.
    ///
    fn refresh(&mut self) -> Result<(), String> {
        // let the old locations_win and view go
        self.locations_view.take();
        match self.weather_data.get_locations(location_filters![]) {
            Ok(locations) => {
                let report = reports::text::Report::default().generate(&locations);
                let view = ReportView::new(report, None).with_show_selected(true).with_active(self.active);
                self.locations_view.replace(LocationsView { locations, view });
                Ok(())
            }
            Err(err) => Err(format!("Locations error ({})", err))?,
        }
    }

    /// Get the size of the tab window.
    ///
    fn size(&self) -> Size {
        self.report_win.as_ref().map_or(Size::default(), |dialog| dialog.size())
    }
    /// Dispatch a key pressed event to the tab window or one of the dialogs. [ControlFlow::Continue] will be
    /// returned if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("LocationsWindow");
        if self.popup.is_some() {
            self.popup_key_pressed(key_event)?;
        } else if self.add_history.is_some() {
            self.add_history_key_pressed(key_event)?;
        } else if self.report_win.is_some() {
            self.history_view_key_pressed(key_event)?;
        }
        if let Some(location_view) = self.locations_view.as_mut() {
            match location_view.view.key_pressed(&key_event) {
                ControlFlow::Break(ControlResult::Continue) => break_event!(DialogResult::Continue)?,
                ControlFlow::Break(ControlResult::Selected(_)) => {
                    self.popup.replace(context_menu::ContextMenu::new());
                    break_event!(DialogResult::Continue)?;
                }
                ControlFlow::Break(ControlResult::NotAllowed) => {
                    beep();
                    break_event!(DialogResult::Continue)?;
                }
                ControlFlow::Continue(()) => (),
                result => log::debug!("Yikes... view result {:?}", result),
            }
        }
        ControlFlow::Continue(())
    }

    /// Draw the tab window and active dialogs on the terminal screen, optionally returning the current
    /// cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the window will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("LocationsWindow");
        let mut coord = None;
        macro_rules! update_coord {
            ($renderer:expr) => {{
                if let Some(render_coord) = $renderer {
                    coord.replace(render_coord);
                }
            }};
        }
        // the location view is first up
        if let Some(locations_view) = &self.locations_view {
            let styles = locations_view.view.catalog_type.get_styles(
                match self.popup.is_some() || self.add_history.is_some() || self.report_win.is_some() {
                    true => ControlState::Normal,
                    false => ControlState::Active,
                },
            );
            update_coord!(locations_view.view.render(area, buffer, styles));
        }
        // show the active dialog
        if let Some(popup) = &self.popup {
            debug_assert!(self.locations_view.is_some(), "render popup bad state\n{:#?}", self);
            let upper_left = match coord.as_ref() {
                None => Position::default(),
                Some(lv_coord) => Position { x: lv_coord.x + 10, y: lv_coord.y },
            };
            update_coord!(popup.render(upper_left, area, buffer));
        } else if let Some(add_history) = &self.add_history {
            // if the position is None, the progress dialog will be running so no cursor
            coord = add_history.render(area, buffer);
        } else if let Some(history_view) = &self.report_win {
            update_coord!(history_view.render(area, buffer));
        }
        coord
    }
}
