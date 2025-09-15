//! The US Cities search UI.

use super::{add_location::AddLocation, alpha, digits};
use crate::cli::reports::list_locations as reports;
use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect, Size},
};
use std::{ops::ControlFlow, rc::Rc};
use termui_lib::prelude::{
    beep, break_event, log_key_pressed, log_render, ok_button, ActiveNormalStyles, Button, ButtonBar, ButtonDialog,
    Control, ControlGroup, ControlResult, ControlState, DialogResult, DialogWindow, EditControl, EditField,
    EditFieldGroup, Label, MessageStyle, ReportView, TextEditor,
};
use weather_lib::prelude::{Location, CityFilter, WeatherData};

/// The add location identifier.
///
const ADD_ID: &'static str = "ADD";

/// The location criteria identifier.
///
const CRITERIA_ID: &'static str = "CRITERIA";

/// The exit dialog identifier.
///
const EXIT_ID: &'static str = "EXIT";

/// The US Cities search and add location manager.
///
pub struct LocationSearch {
    /// The search results dialog.
    dialog: ButtonDialog<SearchWindow>,
    /// The criteria associated with the search results.
    criteria: criteria::CriteriaDialog,
    /// The dialog used to add a weather data location.
    add: Option<AddLocation>,
    /// The weather data API that will be used.
    weather_data: Rc<WeatherData>,
}
impl std::fmt::Debug for LocationSearch {
    /// Debug needs to be implemented because of the weather data API.
    ///
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocationSearch")
            .field("dialog", &self.dialog)
            .field("criteria", &self.criteria)
            .field("add", &self.add)
            .finish()
    }
}
impl LocationSearch {
    /// Create a new instance of the manager.
    ///
    /// # Arguments
    ///
    /// - `weather_data` is the weather data history API that will be used.
    ///
    pub fn new(weather_data: Rc<WeatherData>) -> Self {
        let dialog = ButtonDialog::new(
            ButtonBar::new(vec![
                Button::new(CRITERIA_ID, " Criteria ", 'C').with_active(),
                Button::new(EXIT_ID, " Exit ", 'x'),
            ]),
            SearchWindow::default(),
        )
        .with_title(" US Cities Search ");
        let criteria = criteria::CriteriaDialog::new().with_active();
        Self { dialog, criteria, add: None, weather_data }
    }
    /// Dispatch a key pressed event to the search criteria dialog. [ControlFlow::Continue] will be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    pub fn criteria_key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        match self.criteria.key_pressed(key_event) {
            ControlFlow::Break(DialogResult::Exit) => {
                self.criteria.set_active(false);
                let filter = self.criteria.as_filter();
                match self.weather_data.search_locations(filter) {
                    Ok(locations) => self.dialog.win_mut().initialize(locations),
                    Err(err) => {
                        self.dialog.set_message(MessageStyle::Error, format!("City search failed ({}).", err));
                    }
                }
                break_event!(DialogResult::Continue)
            }
            ControlFlow::Break(DialogResult::Cancel) => {
                self.criteria.set_active(false);
                match self.dialog.win().locations_view.is_none() {
                    true => break_event!(DialogResult::Cancel)?,
                    false => break_event!(DialogResult::Continue)?,
                }
            }
            result => result,
        }
    }
    /// Dispatch a key pressed event to the search dialog. [ControlFlow::Continue] will be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    pub fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("LocationSearch");
        if self.criteria.is_active() {
            self.criteria_key_pressed(key_event)?;
        } else if let Some(mut add) = self.add.take() {
            match add.key_pressed(key_event) {
                ControlFlow::Break(DialogResult::Cancel) => break_event!(DialogResult::Continue)?,
                ControlFlow::Continue(()) => {
                    self.add.replace(add);
                }
                ControlFlow::Break(DialogResult::Continue) => {
                    self.add.replace(add);
                    break_event!(DialogResult::Continue)?;
                }
                result => result?,
            }
        }
        match self.dialog.key_pressed(key_event) {
            ControlFlow::Break(DialogResult::Selected(button_id)) => match button_id.as_str() {
                ADD_ID => {
                    let win = self.dialog.win();
                    debug_assert!(win.locations_view.is_some(), "Add bad state\n{:#?}", self);
                    let location = win.locations_view.as_ref().unwrap().selected_location();
                    let add_dialog = AddLocation::new(self.weather_data.clone()).with_location(location);
                    self.add.replace(add_dialog);
                    break_event!(DialogResult::Continue)?;
                }
                CRITERIA_ID => {
                    self.criteria.set_active(true);
                    break_event!(DialogResult::Continue)?;
                }
                EXIT_ID => {
                    break_event!(DialogResult::Exit)?;
                }
                _ => unreachable!(),
            },
            result => result?,
        }
        ControlFlow::Continue(())
    }
    /// Draw the search dialog on the terminal screen and optionally return the current cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the window will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    pub fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("LocationSearch");
        let mut coord = self.dialog.render(area, buffer);
        if let Some(add) = self.add.as_ref() {
            debug_assert!(!self.criteria.is_active(), "render bad state criteria active\n{:#?}", self);
            if let Some(add_coord) = add.render(area, buffer) {
                coord.replace(add_coord);
            }
        } else if self.criteria.is_active() {
            debug_assert!(self.add.is_none(), "render bad state, add active\n{:#?}", self);
            if let Some(criteria_coord) = self.criteria.render(area, buffer) {
                coord.replace(criteria_coord);
            }
        }
        coord
    }
}

/// The results of a locations search.
#[derive(Debug)]
struct LocationsView {
    /// The collection of locations found by a search.
    locations: Vec<Location>,
    /// The report view associated with found locations.
    view: ReportView,
}
impl LocationsView {
    /// Return the location information associated with the current search row.
    ///
    fn selected_location(&self) -> &Location {
        let selected_row = self.view.selected_row();
        debug_assert!(selected_row < self.locations.len(), "View selected row oob. {:?}", self);
        self.locations.get(selected_row).unwrap()
    }
}

/// The search results window manager.
///
#[derive(Debug, Default)]
struct SearchWindow {
    /// Indicates if the dialog window is active or not.
    active: bool,
    /// The result of a locations search.
    locations_view: Option<LocationsView>,
}
impl SearchWindow {
    /// Updates the search results with a new collection of locations.
    ///
    /// # Arguments
    ///
    /// - `locations` provides the contents of the search results window.
    ///
    fn initialize(&mut self, locations: Vec<Location>) {
        let report = reports::text::Report::default().with_skip_alias().generate(&locations);
        let view = ReportView::new(report, None).with_show_selected(true);
        self.locations_view.replace(LocationsView { locations, view });
    }
}
impl DialogWindow for SearchWindow {
    /// Query if the window is active or not.
    ///
    fn is_active(&self) -> bool {
        self.active
    }
    /// Control if the results view is active or not.
    ///
    /// # Arguments
    ///
    /// - `yes_no` determines if the results view is active or not.
    ///
    fn set_active(&mut self, yes_no: bool) {
        self.active = yes_no;
    }
    /// Get the size of the view.
    ///
    fn size(&self) -> Size {
        self.locations_view.as_ref().map_or(Size { width: 40, height: 5 }, |locations_view| locations_view.view.size())
    }
    /// Dispatch a key pressed event to the results view. [ControlFlow::Continue] will be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("SearchWindow");
        match self.locations_view.as_mut() {
            None => {
                debug_assert!(false, "key_pressed locations view missing\n{:#?}", self);
                log::debug!("SearchDialog locations view is None");
            }
            Some(locations_view) => match locations_view.view.key_pressed(&key_event) {
                ControlFlow::Break(ControlResult::Continue) => break_event!(DialogResult::Continue)?,
                ControlFlow::Break(ControlResult::Selected(_)) => {
                    break_event!(DialogResult::Selected(ADD_ID.to_string()))?
                }
                ControlFlow::Break(ControlResult::NotAllowed) => {
                    beep();
                    break_event!(DialogResult::Continue)?;
                }
                _ => (),
            },
        }
        ControlFlow::Continue(())
    }
    /// Draw the results view on the terminal screen and optionally return the current cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the window will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("SearchWindow");
        // it's normal for the view to be None when first starting the search
        self.locations_view.as_ref().map_or(None, |locations_view| {
            let styles = locations_view.view.catalog_type.get_styles(ControlState::Active);
            locations_view.view.render(area, buffer, styles)
        })
    }
}

mod criteria {
    //! The search criteria dialog
    use super::*;

    /// The full state edit field identifier.
    ///
    const CITY_ID: &'static str = "CITY";

    /// The two-letter state edit field identifier.
    ///
    const STATE_ID: &'static str = "STATE_ID";

    /// The name edit field identifier.
    ///
    const ZIP_CODE_ID: &'static str = "ZIP_CODE";

    /// The limit edit field identifier.
    ///
    const LIMIT_ID: &'static str = "LIMIT";

    /// The search criteria [button dialog](ButtonDialog) manager.
    #[derive(Debug)]
    pub struct CriteriaDialog {
        /// The search criteria [button dialog](ButtonDialog).
        dialog: ButtonDialog<CriteriaWindow>,
    }
    impl CriteriaDialog {
        /// Create a new instance of the dialog.
        pub fn new() -> Self {
            let buttons = ButtonBar::new(vec![ok_button().with_active()]).with_auto_select(true);
            let window = CriteriaWindow::new();
            let dialog = ButtonDialog::new(buttons, window).with_title(" US Cities Criteria ");
            Self { dialog }
        }
        /// A builder method used to set the dialog active.
        ///
        pub fn with_active(mut self) -> Self {
            self.set_active(true);
            self
        }
        /// Query if the dialog is active or not.
        ///
        pub fn is_active(&self) -> bool {
            self.dialog.win().is_active()
        }
        /// Controls if the dialog is active or not.
        ///
        /// # Arguments
        ///
        /// - `yes_no` determines if the results view is active or not.
        ///
        pub fn set_active(&mut self, yes_no: bool) {
            self.dialog.win_mut().set_active(yes_no);
        }
        /// Get the search criteria from the window.
        ///
        pub fn as_filter(&self) -> CityFilter {
            self.dialog.win().as_filter_and_limit()
        }
        /// Dispatch a key pressed event to the dialog. [ControlFlow::Continue] will be returned if the
        /// event is not consumed.
        ///
        /// # Arguments
        ///
        /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
        ///
        pub fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
            log_key_pressed!("CriteriaDialog");
            match self.dialog.key_pressed(key_event) {
                ControlFlow::Break(DialogResult::Cancel) => break_event!(DialogResult::Cancel),
                ControlFlow::Break(DialogResult::Selected(_)) => break_event!(DialogResult::Exit),
                ControlFlow::Break(DialogResult::Continue) => break_event!(DialogResult::Continue),
                result => {
                    debug_assert!(false, "missed event {:?}", result);
                    result
                }
            }
        }
        /// Draw the dialog on the terminal screen and optionally return the current cursor position.
        ///
        /// # Arguments
        ///
        /// - `area` is where on the terminal screen the window will be drawn.
        /// - `buffer` is the current view of the terminal screen.
        ///
        pub fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
            log_render!("CriteriaDialog");
            self.dialog.render(area, buffer)
        }
    }

    /// The criteria edit fields manager.
    #[derive(Debug)]
    struct CriteriaWindow {
        /// Indicates one of the edit fields are currently active.
        active: bool,
        /// The group of edit fields.
        fields: EditFieldGroup,
    }
    impl CriteriaWindow {
        /// Create a new instance of the edit fields manager.
        ///
        fn new() -> Self {
            Self {
                active: false,
                fields: EditFieldGroup::new(vec![
                    EditField::new(
                        Label::align_right("City: ").with_selector('C').with_id(CITY_ID).with_active(),
                        TextEditor::default().with_valid_chars(alpha().chain(" *".chars())).with_width(25),
                    ),
                    EditField::new(
                        Label::align_right("State: ").with_selector('S').with_id(STATE_ID),
                        TextEditor::default().with_valid_chars(alpha().chain(" *".chars())).with_width(25),
                    ),
                    EditField::new(
                        Label::align_right("Zip Code: ").with_selector('N').with_id(ZIP_CODE_ID),
                        TextEditor::default().with_valid_chars(digits().chain("*".chars())).with_width(6),
                    ),
                    EditField::new(
                        Label::align_right("Limit: ").with_selector('L').with_id(LIMIT_ID),
                        TextEditor::default().with_valid_chars(digits()).with_width(3).with_text(25),
                    ),
                ]),
            }
        }
        /// Transform the edit fields into the filter criteria used by a location search.
        fn as_filter_and_limit(&self) -> CityFilter {
            let mut filter = CityFilter::default();

            let city = self.fields.get(CITY_ID).unwrap().text();
            if !city.is_empty() {
                filter.name = Some(city.into());
            }

            let state = self.fields.get(STATE_ID).unwrap().text();
            if !state.is_empty() {
                filter.state = Some(state.into());
            }

            let zip_code = self.fields.get(ZIP_CODE_ID).unwrap().text();
            if !zip_code.is_empty() {
                filter.zip_code = Some(zip_code.into());
            }

            filter.limit = self.fields.get(LIMIT_ID).unwrap().text().parse().unwrap_or(25);
            filter
        }
    }
    impl DialogWindow for CriteriaWindow {
        /// Query if the dialog is active or not.
        ///
        fn is_active(&self) -> bool {
            self.active
        }
        /// Control if the dialog is active or not.
        ///
        /// # Arguments
        ///
        /// - `yes_no` determines if the dialog is active or not.
        ///
        fn set_active(&mut self, yes_no: bool) {
            self.active = yes_no;
        }
        /// Get the size of the dialog.
        ///
        fn size(&self) -> Size {
            self.fields.size()
        }
        /// Dispatch a key pressed event to the dialog. [ControlFlow::Continue] will be returned if the
        /// event is not consumed.
        ///
        /// # Arguments
        ///
        /// - `key_event` is guaranteed to be a [key pressed](crossterm::event::KeyEventKind::Press) event.
        ///
        fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
            log_key_pressed!("CriteriaWindow");
            if let ControlFlow::Break(control_result) = self.fields.key_pressed(key_event) {
                match control_result {
                    ControlResult::NotAllowed | ControlResult::NextGroup | ControlResult::PrevGroup => {
                        beep();
                        break_event!(DialogResult::Continue)?;
                    }
                    ControlResult::Continue => break_event!(DialogResult::Continue)?,
                    _ => (),
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
        fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
            log_render!("CriteriaWindow");
            self.fields.render(area, buffer, ActiveNormalStyles::new(self.fields.catalog_type))
        }
    }
}
