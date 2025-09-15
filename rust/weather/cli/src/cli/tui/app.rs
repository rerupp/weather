//! The weather data UI.
use super::{dialogs, histories_win, locations_win, summary_win};
use crate::cli;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use std::{ops::ControlFlow, rc::Rc};
use termui_lib::prelude::{
    break_event, log_key_pressed, log_render, Application, ApplicationResult, Console, DialogResult, DialogWindow,
    MenuDialog, MenuItem, Menubar, MessageStyle, TabDialog, TabWindow,
};
use weather_lib::prelude::WeatherData;

/// The UI runner.
pub fn run(weather_data: WeatherData) -> cli::Result<()> {
    let app = app::WeatherApp::new(weather_data);
    let mut console = Console::new()?;
    console.run(app)?;
    Ok(())
}

mod app {
    //! The current version of the weather data TUI.
    use super::*;
    use dialogs::{AddLocation, LocationSearch};
    use histories_win::HistoriesWindow;
    use locations_win::LocationsWindow;
    use summary_win::SummaryWindow;

    /// The locations_win window identifier.
    const LOCATIONS_WIN_ID: &'static str = "LOCATIONS";
    /// The summary information window identifier.
    const SUMMARY_WIN_ID: &'static str = "SUMMARY";
    /// The history information window identifier.
    const HISTORY_WIN_ID: &'static str = "HISTORY";
    /// The main menu new action identifier.
    const NEW_ID: &'static str = "NEW";
    /// The submenu search locations_win identifier.
    const SEARCH_ID: &'static str = "SEARCH";
    /// The main menu view action identifier.
    const VIEW_ID: &'static str = "VIEW";
    /// The main menu search locations_win identifier.
    const USCITIES_ID: &'static str = "USCITIES";
    /// The main menu add location identifier.
    const ADD_LOCATION_ID: &'static str = "ADD";
    /// The main menu exit identifier.
    const EXIT_ID: &'static str = "EXIT";

    /// The weather data TUI application.
    pub struct WeatherApp {
        /// The main window is a [menu dialog](MenuDialog) with a [tab dialog](TabDialog) window providing
        /// multiple views of weather data.
        dialog: MenuDialog<TabDialog>,
        /// A dialog to add locations.
        add: Option<AddLocation>,
        /// A dialog to add locations from the US Cities DB.
        search: Option<LocationSearch>,
        /// The backend weather data API.
        weather_data: Rc<WeatherData>,
    }
    impl std::fmt::Debug for WeatherApp {
        /// Show the contents bypassing the weather data API instance.
        ///
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ConsoleApp")
                .field("dialog", &self.dialog)
                .field("add", &self.add)
                .field("search", &self.search)
                .finish()
        }
    }
    impl WeatherApp {
        /// Create a new instance of the console application.
        ///
        /// # Arguments
        ///
        /// - `weather_data` is the weather history API that will be used.
        ///
        pub fn new(weather_data: WeatherData) -> Self {
            let weather_data = Rc::new(weather_data);
            let dialog = MenuDialog::new(
                Menubar::new(vec![
                    MenuItem::new(NEW_ID, "New", 'N').with_menu(vec![
                        MenuItem::new(ADD_LOCATION_ID, "Manual", 'M').with_char_select(),
                        MenuItem::new(SEARCH_ID, "Search", 'S').with_char_select().with_menu(vec![MenuItem::new(
                            USCITIES_ID,
                            "US Cities",
                            'U',
                        )
                        .with_char_select()]),
                    ]),
                    MenuItem::new(VIEW_ID, "View", 'V').with_menu(vec![
                        MenuItem::new(LOCATIONS_WIN_ID, "Location", 'L').with_char_select(),
                        MenuItem::new(SUMMARY_WIN_ID, "Summary", 'S').with_char_select(),
                        MenuItem::new(HISTORY_WIN_ID, "Histories", 'H').with_char_select(),
                    ]),
                    MenuItem::new(EXIT_ID, "Exit", 'x'),
                ]),
                TabDialog::new(),
            );
            let mut self_ = Self { weather_data: weather_data.clone(), dialog, add: None, search: None };
            self_.show_locations();
            self_
        }

        /// Add the [locations window](LocationsWindow) to the tab dialog.
        ///
        fn show_locations(&mut self) {
            match self.dialog.win().contains_tab(LOCATIONS_WIN_ID) {
                true => self.dialog.win_mut().set_active_tab(LOCATIONS_WIN_ID),
                false => match LocationsWindow::new(self.weather_data.clone()) {
                    Ok(win) => {
                        let tab = TabWindow::new(LOCATIONS_WIN_ID, "Locations", 'L', win);
                        self.dialog.win_mut().add_or_replace_tab(tab);
                    }
                    Err(error_msg) => self.dialog.set_message(MessageStyle::Error, error_msg),
                },
            }
        }

        /// Add the [summary window](SummaryWindow) to the tab dialog.
        ///
        fn show_summary(&mut self) {
            match self.dialog.win().contains_tab(SUMMARY_WIN_ID) {
                true => self.dialog.win_mut().set_active_tab(SUMMARY_WIN_ID),
                false => match SummaryWindow::new(self.weather_data.clone()) {
                    Ok(win) => {
                        let tab = TabWindow::new(SUMMARY_WIN_ID, "Summary", 'S', win);
                        self.dialog.win_mut().add_or_replace_tab(tab);
                    }
                    Err(error_msg) => self.dialog.set_message(MessageStyle::Error, error_msg),
                },
            }
        }

        /// Add the [histories window](HistoriesWindow) to the tab dialog.
        ///
        fn show_histories(&mut self) {
            match self.dialog.win().contains_tab(HISTORY_WIN_ID) {
                true => self.dialog.win_mut().set_active_tab(HISTORY_WIN_ID),
                false => match HistoriesWindow::new(self.weather_data.clone()) {
                    Ok(win) => {
                        let tab = TabWindow::new(HISTORY_WIN_ID, "Histories", 'H', win);
                        self.dialog.win_mut().add_or_replace_tab(tab);
                    }
                    Err(error_msg) => self.dialog.set_message(MessageStyle::Error, error_msg),
                },
            }
        }

        /// Give the [menu dialog](Self::dialog) a chance to consume the event.
        /// [ControlFlow::Continue] will be returned if the event is not consumed.
        ///
        /// # Arguments
        ///
        /// - `key_event` is guaranteed to be a KeyEventKind::Press event.
        ///
        fn app_key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<ApplicationResult> {
            log_key_pressed!("App");
            // check to see if you should refresh the screen
            if key_event.modifiers == KeyModifiers::NONE && key_event.code == KeyCode::F(5) {
                if let Err(error) = self.dialog.win_mut().refresh() {
                    self.dialog.set_message(MessageStyle::Error, format!("Refresh error ({})", error));
                }
            } else {
                if let ControlFlow::Break(dialog_result) = self.dialog.key_pressed(key_event) {
                    match dialog_result {
                        DialogResult::Exit => break_event!(ApplicationResult::Exit)?,
                        DialogResult::Cancel => {
                            let tab_dialog = self.dialog.win_mut();
                            tab_dialog.remove_active_tab();
                            match tab_dialog.is_empty() {
                                true => break_event!(ApplicationResult::Exit)?,
                                false => {
                                    let _ = tab_dialog.refresh();
                                }
                            }
                        }
                        DialogResult::Selected(id) => {
                            self.dialog.reset_menu();
                            match id.as_str() {
                                ADD_LOCATION_ID => {
                                    debug_assert!(self.add.is_none(), "add already active\n{:#?}", self);
                                    debug_assert!(self.search.is_none(), "search is active\n{:#?}", self);
                                    self.add.replace(AddLocation::new(self.weather_data.clone()));
                                }
                                USCITIES_ID => {
                                    debug_assert!(self.search.is_none(), "search already active\n{:#?}", self);
                                    debug_assert!(self.add.is_none(), "add is active\n{:#?}", self);
                                    self.search.replace(LocationSearch::new(self.weather_data.clone()));
                                }
                                LOCATIONS_WIN_ID => self.show_locations(),
                                SUMMARY_WIN_ID => self.show_summary(),
                                HISTORY_WIN_ID => self.show_histories(),
                                EXIT_ID => {
                                    break_event!(ApplicationResult::Exit)?;
                                }
                                _ => unreachable!(),
                            }
                        }
                        DialogResult::Error(error) => self.dialog.set_message(MessageStyle::Error, error),
                        DialogResult::Poll(poll_ms) => break_event!(ApplicationResult::Poll(poll_ms))?,
                        DialogResult::Continue => (),
                    }
                }
            }
            ControlFlow::Continue(())
        }
    }
    impl Application for WeatherApp {
        /// Dispatch a key pressed event to the [menu dialog](Self::dialog). [Continue](ControlFlow::Continue) will
        /// be returned if the event is not consumed.
        ///
        /// # Arguments
        ///
        /// - `key_event` is guaranteed to be a KeyEventKind::Press event.
        ///
        fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<ApplicationResult> {
            if let Some(mut add) = self.add.take() {
                match add.key_pressed(key_event) {
                    ControlFlow::Break(DialogResult::Cancel) => (),
                    ControlFlow::Break(DialogResult::Exit) => {
                        // a location was added so refresh the window
                        if let Err(error_msg) = self.dialog.win_mut().refresh() {
                            self.dialog.set_message(MessageStyle::Error, error_msg);
                        }
                    }
                    _ => {
                        self.add.replace(add);
                    }
                }
            } else if let Some(mut search) = self.search.take() {
                match search.key_pressed(key_event) {
                    ControlFlow::Break(DialogResult::Cancel) => (),
                    ControlFlow::Break(DialogResult::Exit) => {
                        // a location was added so refresh the window
                        if let Err(error_msg) = self.dialog.win_mut().refresh() {
                            self.dialog.set_message(MessageStyle::Error, error_msg);
                        }
                    }
                    _ => {
                        self.search.replace(search);
                    }
                }
            } else {
                self.app_key_pressed(key_event)?;
            }
            ControlFlow::Continue(())
        }

        /// Draw the application on the terminal screen.
        ///
        /// # Arguments
        ///
        /// - `frame` contains a current view into the terminal state.
        ///
        fn render(&self, frame: &mut Frame) {
            log_render!("App");
            let area = frame.area();
            let mut coord = self.dialog.render(area, frame.buffer_mut());
            if let Some(search) = &self.search {
                if let Some(search_coord) = search.render(area, frame.buffer_mut()) {
                    coord.replace(search_coord);
                }
            } else if let Some(add) = &self.add {
                if let Some(add_coord) = add.render(area, frame.buffer_mut()) {
                    coord.replace(add_coord);
                }
            }
            if let Some(position) = coord {
                frame.set_cursor_position(position);
            }
        }
    }
}
