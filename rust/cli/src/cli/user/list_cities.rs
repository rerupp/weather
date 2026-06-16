//! The user command list city locations.
//!
//! usage: lc [*CITY*][, *STATE*] [+*COUNTRY*]

use super::location_filters;
use crate::cli::{self, get_writer, reports::list_locations as reports, user::trim_row_end, ReportArgs};
use clap::{Arg, ArgAction, ArgMatches, Command};
use tui_lib::run_viewport;
use weather_lib::prelude::{Location, WeatherData};

/// The list cities command name.
///
pub const COMMAND_NAME: &'static str = "lc";

/// The default search result limit argument id.
///
const LIMIT: &str = "LIMIT";

/// The terminal UI command argument id.
///
const TUI: &str = "TUI";

/// Create the list cities command.
///
pub fn command() -> Command {
    // get the size of the screen to determine the default limit
    let (_, height) = crossterm::terminal::size().unwrap_or((80, 24));
    let mut tui_conflicts = vec![LIMIT];
    tui_conflicts.append(&mut ReportArgs::arg_ids());
    Command::new(COMMAND_NAME)
        .about("Show city metadata that can be used to create a new location.")
        .arg(
            Arg::new(TUI)
                .short('t')
                .long("tui")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(tui_conflicts)
                .help("Search cities interactively."),
        )
        .arg(
            Arg::new(LIMIT)
                .short('l')
                .long("limit")
                .action(ArgAction::Set)
                .value_name("LIMIT")
                .require_equals(true)
                .value_parser(limit_parser)
                .default_value(format!("{}", height - 3))
                .help("Limit the number of cities that will be returned."),
        )
        .args(ReportArgs::get())
        .group(ReportArgs::arg_group())
        .arg(location_filters::arg())
}

/// Execute the list cities command.
///
/// # Arguments
///
/// * `weather_data` is the weather history API that will be used.
/// * `args` holds the command line arguments.
///
pub fn execute(weather_data: &WeatherData, args: ArgMatches) -> cli::Result<()> {
    match args.get_flag(TUI) {
        false => report(weather_data, &args),
        true => {
            let query = location_filters::get_query_str(&args).unwrap_or_default();
            let mut list_cities = tui::ListCities::new(weather_data, query);
            run_viewport(list_cities.viewport_rows, |terminal| list_cities.run(terminal))?;
            if let Some(location) = list_cities.added_location {
                println!("{location} was added.");
            }
            Ok(())
        }
    }
}

fn report(weather_data: &WeatherData, args: &ArgMatches) -> cli::Result<()> {
    let filters = location_filters::parse_args(&args)?;
    let limit = *args.get_one::<usize>(LIMIT).unwrap();
    let locations =
        weather_data.get_cities(filters, limit)?.into_iter().map(|city| Location::from(city)).collect::<Vec<_>>();
    match locations.is_empty() {
        true => println!("There were no cities found."),
        false => {
            let report_args = ReportArgs::new(&args);
            let mut writer = get_writer(&report_args)?;
            let report = if report_args.csv() {
                reports::csv::Report::default().generate(locations)
            } else if report_args.json() {
                let report = match report_args.pretty() {
                    true => reports::json::Report::pretty_printed(),
                    false => reports::json::Report::default(),
                };
                report.generate(locations)
            } else {
                reports::text::Report::default()
                    .with_skip_alias()
                    .with_title_separator()
                    .generate(&locations)
                    .into_iter()
                    .map(|row| trim_row_end!(row.to_string()))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            if let Err(error) = writer.write_all(report.as_bytes()) {
                cli::err!("There was an error writing the cities report: {:?}", error)?;
            }
        }
    }
    Ok(())
}

/// Used by the command parser to make sure the limit is within bounds.
///
/// # Arguments
///
/// * `limit_arg` is the weather directory command argument.
///
fn limit_parser(limit_arg: &str) -> Result<usize, String> {
    match limit_arg.parse::<usize>() {
        Err(_) => Err("limit needs to be an unsigned integer.".to_string()),
        Ok(limit) => match limit > 0 {
            false => Err("limit must be greater than 0".to_string()),
            true => Ok(limit),
        },
    }
}

mod tui {
    //! An inline terminal UI to search the cities DB.

    use super::{
        super::location::tui::{LocationEditor, LocationEditorMode},
        location_filters, reports,
    };
    use crate::cli;
    use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    use ratatui::{
        prelude::{Alignment, Frame, Line, Position, Rect, Style, Widget},
        widgets::Clear,
        DefaultTerminal,
    };
    use std::ops::ControlFlow;
    use tui_lib::{beep, CommandBar, CommandBarStyles, Editor, EditorResult, EditorStyles, Label};
    use weather_lib::prelude::{Location, LocationFilter, WeatherData};

    /// The result of running the location editor.
    ///
    enum AddCityResult {
        /// The [LocationEditor] was canceled.
        Canceled,
        /// The [LocationEditor] returned a new location.
        Added(Location),
    }

    /// An inline TUI view of available cities.
    ///
    pub struct ListCities<'lc> {
        /// The weather data API that is used
        weather_data: &'lc WeatherData,
        /// The city views current row.
        view_row: usize,
        /// The number of cities shown in the view.
        max_view_row: usize,
        /// The overall size of the TUI.
        pub viewport_rows: u16,
        /// The current cursor position.
        position: Option<Position>,
        /// Draw the terminal screen if there are updates.
        redraw: bool,
        /// The area on the screen where the view was last drawn.
        screen_area: Rect,
        /// The width of the list cities report.
        report_width: u16,
        /// The report header rows.
        titles: Vec<String>,
        /// The collection of city rows found by the query.
        cities: Vec<(String, Location)>,
        /// A location that might have been added.
        pub added_location: Option<Location>,
        /// The query editor.
        query: Editor,
        /// Problems with the location properties.
        problems: Option<String>,
        /// The command bar for list cities.
        command_bar: CommandBar,
    }
    impl<'lc> ListCities<'lc> {
        /// Create a new instance of the list cities TUI.
        ///
        /// # Arguments
        ///
        /// * `weather_data` is the weather history data API that will be used.
        /// * `query` will be used to initially populate the city query.
        ///
        pub fn new(weather_data: &'lc WeatherData, query: String) -> Self {
            let (_, height) = crossterm::terminal::size().unwrap_or((80, 24));
            Self {
                weather_data,
                view_row: 0,
                max_view_row: 0,
                viewport_rows: height,
                position: None,
                redraw: false,
                screen_area: Rect::default(),
                report_width: 0,
                titles: vec![],
                cities: vec![],
                added_location: None,
                query: Editor::new(query)
                    .with_width(40)
                    .with_valid_chars(('A'..='Z').chain('a'..='z').chain('0'..='9').chain(" -_,+*.".chars()))
                    .with_label(Label::new("Query: ").with_alignment(Alignment::Right)),
                problems: None,
                command_bar: CommandBar::default()
                    .add_command("^Q", " to change the query")
                    .add_command("ENTER", " to add a city")
                    .add_command("ESC", " to quit")
                    .with_alignment(Alignment::Center)
                    .with_styles(CommandBarStyles::common()),
            }
        }

        /// Manage the terminal screen drawing the editor contents and dispatching keystrokes.
        ///
        /// # Arguments
        ///
        /// * `terminal` is used to show the view on the screen.
        ///
        pub fn run(&mut self, terminal: &mut DefaultTerminal) -> cli::Result<()> {
            self.generate_report()?;
            self.set_query_active(self.problems.is_some() || self.cities.is_empty());
            self.redraw = true;
            loop {
                // draw the screen
                if self.redraw {
                    if let Err(error) = terminal.draw(|frame| self.draw(frame)) {
                        cli::err!("There was a problem drawing the screen: {error}")?;
                    }
                    // show the cursor
                    if let Some(position) = self.position {
                        terminal.set_cursor_position(position).unwrap();
                        terminal.show_cursor().unwrap();
                    }
                    self.redraw = false;
                }

                // process the next event
                match event::read() {
                    Err(error) => cli::err!("There was a problem reading the keyboard: {error}")?,
                    Ok(Event::Mouse(mouse_event)) => self.mouse_event(mouse_event),
                    Ok(Event::Key(key_event)) => {
                        if self.key_event(key_event, terminal)?.is_break() {
                            break;
                        }
                    }
                    _ => (),
                }
            }
            Ok(())
        }

        /// Manage key events for the city view.
        ///
        /// # Arguments
        ///
        /// * `key_event` is the key event that will be processed.
        /// * `terminal` is used if a city is being added as a new location.
        ///
        fn key_event(&mut self, key_event: KeyEvent, terminal: &mut DefaultTerminal) -> cli::Result<ControlFlow<()>> {
            let mut flow = ControlFlow::Continue(());
            // only manage a key press event
            if key_event.is_press() {
                // by default redraw the screen
                self.redraw = true;

                // beep the terminal when a key event was not allowed and do not redraw the screen
                macro_rules! beep {
                    () => {{
                        beep();
                        self.redraw = false;
                    }};
                }
                match (key_event.modifiers, key_event.code) {
                    (KeyModifiers::NONE, KeyCode::Esc) => flow = ControlFlow::Break(()),
                    (KeyModifiers::NONE, KeyCode::Up) => match self.view_row == 0 {
                        true => beep!(),
                        false => self.view_row -= 1,
                    },
                    (KeyModifiers::NONE, KeyCode::Down) => match self.view_row < self.max_view_row - 1 {
                        false => beep!(),
                        true => self.view_row += 1,
                    },
                    // change the query
                    (KeyModifiers::CONTROL, KeyCode::Char(key)) => match key == 'Q' || key == 'q' {
                        false => beep!(),
                        true => self.set_query_active(true),
                    },
                    (KeyModifiers::NONE, KeyCode::Enter) => match self.query.is_active() {
                        // generate a report using the query text
                        true => {
                            self.generate_report()?;
                            self.set_query_active(self.problems.is_some() || self.cities.is_empty());
                            self.view_row = 0;
                        }
                        // add a new location from the current row
                        false => {
                            let (_, city) = self.cities.get(self.view_row).unwrap();
                            if let AddCityResult::Added(location) = self.run_add_location(city, terminal)? {
                                self.added_location.replace(location);
                                flow = ControlFlow::Break(());
                            }
                        }
                    },
                    // only pass the key onto the editor if it is active
                    _ => match self.query.is_active() {
                        false => beep!(),
                        true => match self.query.key_pressed(&key_event) {
                            EditorResult::Consumed => (),
                            _ => beep!(),
                        },
                    },
                }
            }
            Ok(flow)
        }

        /// Manage mouse events for the view.
        ///
        /// # Arguments
        ///
        /// * `mouse_event` is the event.
        ///
        fn mouse_event(&mut self, mouse_event: MouseEvent) {
            // only deal with left button mouse clicks
            if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                // make sure the click was within the screen area
                if self.screen_area.contains(Position { x: mouse_event.column, y: mouse_event.row }) {
                    // was the query clicked?
                    match self.screen_area.y == mouse_event.row {
                        true => {
                            if !self.query.is_active() {
                                self.set_query_active(true);
                                self.redraw = true;
                            }
                        }
                        false => {
                            // make sure a report row was clicked (skip over the query and title rows)
                            let view_top_row = self.screen_area.y + 1 + self.titles.len() as u16;
                            let view_bottom_row = view_top_row + self.max_view_row as u16;
                            if mouse_event.row >= view_top_row && mouse_event.row < view_bottom_row {
                                // make sure the query is inactive
                                if self.query.is_active() {
                                    self.set_query_active(false);
                                    self.redraw = true;
                                }
                                // make sure the view row has changed
                                let view_row = (mouse_event.row - view_top_row) as usize;
                                if self.view_row != view_row {
                                    self.view_row = view_row;
                                    self.redraw = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        /// Set the state and styles for the query editor.
        ///
        /// # Arguments
        ///
        /// * `active` determines if the editor is active or not.
        ///
        fn set_query_active(&mut self, active: bool) {
            self.query.set_active(active);
            match self.query.is_active() {
                true => self.query.set_styles(EditorStyles::active()),
                false => self.query.set_styles(EditorStyles::dim()),
            }
        }

        /// Draw the contents of the editor onto the terminal screen.
        ///
        /// # Arguments
        ///
        /// * `frame` is the current screen frame that will be used.
        ///
        fn draw(&mut self, frame: &mut Frame) {
            self.screen_area = frame.area();
            let buffer = frame.buffer_mut();

            // clear the viewport
            Clear::default().render(self.screen_area, buffer);

            // the view area is where everything is rendered
            let mut view_area = Rect {
                x: self.screen_area.x,
                y: self.screen_area.y,
                width: std::cmp::max(self.command_bar.width(), self.report_width),
                height: 1,
            };

            // render the query, if it is active the cursor position will be returned
            self.position = self.query.render(view_area, buffer);

            // render any problems
            if let Some(problems) = &self.problems {
                let problems_x = view_area.x + self.query.width() + 1;
                let problems_width = self.screen_area.width - problems_x;
                let problems_area = Rect { x: problems_x, y: view_area.y, width: problems_width, height: 1 };
                Line::styled(problems, Style::default().yellow()).render(problems_area, buffer);
            }
            view_area.y += 1;

            // add the report titles
            let title_style = Style::default().blue();
            for title in &self.titles {
                Line::styled(title, title_style).render(view_area, buffer);
                view_area.y += 1;
            }

            // get the view height and set the max view row
            let view_height = self.screen_area.height.saturating_sub(1 + self.titles.len() as u16 + 1 + 1) as usize;
            self.max_view_row = std::cmp::min(view_height, self.cities.len());

            // show the report contents
            let query_active = self.query.is_active();
            for city_idx in 0..view_height {
                if let Some((row, _)) = self.cities.get(city_idx) {
                    let style = if query_active {
                        Style::default()
                    } else if city_idx == self.view_row {
                        // if the editor did not return a position it needs to be the current row
                        if self.position.is_none() {
                            self.position.replace(view_area.as_position());
                        }
                        Style::default().green()
                    } else {
                        Style::default()
                    };
                    Line::styled(row, style).render(view_area, buffer);
                }
                view_area.y += 1;
            }

            // render the control bar
            self.command_bar.render(view_area, buffer);
        }

        /// Create the text based report that will be shown in the view.
        ///
        fn generate_report(&mut self) -> cli::Result<()> {
            let query = self.query.text();
            match location_filters::parse(query.to_string()) {
                Err(error) => {
                    self.problems.replace(error.to_string());
                }
                // Ok(mut filter) => {
                Ok(filters) => {
                    self.problems.take();
                    // query and retrieve the cities (take into account the banner, problems, and command rows
                    let limit = self.viewport_rows as usize - 3;
                    let locations = self
                        .weather_data
                        // the query editor restricts input so there will never be bad query characters
                        .get_cities(filters, limit)?
                        .into_iter()
                        .map(|city| Location::from(city))
                        .collect::<Vec<_>>();

                    // generate the text report
                    let report_sheet =
                        reports::text::Report::default().with_skip_alias().with_title_separator().generate(&locations);
                    let mut report = report_sheet.into_iter();

                    // there will always be a title and separator unless something is AFU
                    self.titles.clear();
                    self.titles.push(report.next().unwrap().to_string());
                    self.titles.push(report.next().unwrap().to_string());
                    let titles_width = self.titles.iter().map(|t| t.len()).max().unwrap_or(0);

                    // save the cities that were found
                    self.cities.clear();
                    for (sheet_row, city) in report.zip(locations) {
                        self.cities.push((sheet_row.to_string(), city));
                    }
                    let cities_width = self.cities.iter().map(|(c, _)| c.len()).max().unwrap_or(0);
                    self.report_width = std::cmp::max(titles_width, cities_width) as u16;

                    // force the query active if there are no cities
                    if self.cities.is_empty() {
                        self.problems.replace("There were no cities found.".to_string());
                    } else {
                        self.problems.take();
                    }
                }
            }
            Ok(())
        }

        /// Runs the [LocationEditor] to get the alias name and then adds the new location.
        ///
        /// # Arguments
        ///
        /// * `location` has the city properties for the new location.
        /// * `terminal` is used to draw the [LocationEditor] on the screen.
        ///
        fn run_add_location(&self, location: &Location, terminal: &mut DefaultTerminal) -> cli::Result<AddCityResult> {
            let mut editor = LocationEditor::new(LocationEditorMode::Add(location.clone()));
            loop {
                match editor.run(terminal)? {
                    None => return Ok(AddCityResult::Canceled),
                    Some(location) => {
                        let filters = vec![LocationFilter::alias(&location.alias)];
                        let matches = self.weather_data.get_locations(Some(filters))?;
                        if matches.len() > 0 {
                            editor.set_problems("The alias name is already being used.");
                        } else {
                            self.weather_data.add_location(location.clone())?;
                            return Ok(AddCityResult::Added(location));
                        }
                    }
                }
            }
        }
    }
}
