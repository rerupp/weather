//! This module has the common code used by add and modify.

pub mod cmd {
    //! The cli command definition and argument option utilities.

    use clap::{Arg, ArgAction, ArgMatches, Command};
    use weather_lib::prelude::Location;

    /// The command argument id for using an interactive mode.
    const TUI: &'static str = "TUI";

    /// The command argument id for the alias name.
    const ALIAS: &'static str = "ALIAS";

    /// The command argument id for the state name.
    const COUNTRY_NAME: &'static str = "COUNTRY_NAME";
    /// The other arguments required when [COUNTRY_NAME] is used in conjunction with [crate::cli::user::location_args::CommandMode::Add].
    const COUNTRY_NAME_REQS: [&str; 7] = [COUNTRY_CODE, REGION_NAME, REGION_CODE, CITY_NAME, LATITUDE, LONGITUDE, TZ];

    /// The command argument id for the region code.
    const COUNTRY_CODE: &'static str = "COUNTRY_CODE";
    /// The other arguments required when [COUNTRY_CODE] is used in conjunction with [crate::cli::user::location_args::CommandMode::Add].
    const COUNTRY_CODE_REQS: [&str; 7] = [COUNTRY_NAME, REGION_NAME, REGION_CODE, CITY_NAME, LATITUDE, LONGITUDE, TZ];

    /// The command argument id for the state name.
    const REGION_NAME: &'static str = "REGION_NAME";
    /// The other arguments required when [REGION_NAME] is used in conjunction with [crate::cli::user::location_args::CommandMode::Add].
    const REGION_NAME_REQS: [&str; 7] = [COUNTRY_NAME, COUNTRY_CODE, REGION_CODE, CITY_NAME, LATITUDE, LONGITUDE, TZ];

    /// The command argument id for the region code.
    const REGION_CODE: &'static str = "REGION_CODE";
    /// The other arguments required when [REGION_CODE] is used in conjunction with [crate::cli::user::location_args::CommandMode::Add].
    const REGION_CODE_REQS: [&str; 7] = [COUNTRY_NAME, COUNTRY_CODE, REGION_NAME, CITY_NAME, LATITUDE, LONGITUDE, TZ];

    /// The command argument id for the city name.
    const CITY_NAME: &'static str = "CITY_NAME";
    /// The other arguments required when [CITY_NAME] is used in conjunction with [crate::cli::user::location_args::CommandMode::Add].
    const CITY_NAME_REQS: [&str; 7] = [COUNTRY_NAME, COUNTRY_CODE, REGION_NAME, REGION_CODE, LATITUDE, LONGITUDE, TZ];

    /// The command argument id for the latitude.
    const LATITUDE: &'static str = "LATITUDE";
    /// The other arguments required when [LATITUDE] is used in conjunction with [crate::cli::user::location_args::CommandMode::Add].
    const LATITUDE_REQS: [&str; 7] = [COUNTRY_NAME, COUNTRY_CODE, REGION_NAME, REGION_CODE, CITY_NAME, LONGITUDE, TZ];

    /// The command argument id for the longitude.
    const LONGITUDE: &'static str = "LONGITUDE";
    /// The other arguments required when [LONGITUDE] is used in conjunction with [crate::cli::user::location_args::CommandMode::Add].
    const LONGITUDE_REQS: [&str; 7] = [COUNTRY_NAME, COUNTRY_CODE, REGION_NAME, REGION_CODE, CITY_NAME, LATITUDE, TZ];

    /// The command argument id for the longitude.
    const TZ: &'static str = "TZ";
    /// The other arguments required when [TZ] is used in conjunction with [crate::cli::user::location_args::CommandMode::Add].
    const TZ_REQS: [&str; 7] = [COUNTRY_NAME, COUNTRY_CODE, REGION_NAME, REGION_CODE, CITY_NAME, LATITUDE, LONGITUDE];

    /// The command mode determines the configuration and behavior of the resulting CLI command.
    #[derive(Eq, PartialEq, Debug)]
    pub enum CommandMode {
        /// The command will be configured for adding a location.
        Add,
        /// The command will be configured for modifying a location.
        Modify,
    }

    /// Create a CLI command tailored to either add or modify.
    ///
    /// # Arguments
    ///
    /// * `name` is the name of the CLI command.
    /// * `about` is the short help description.
    /// * `mode` determines if the command will be used for add or modify.
    ///
    pub fn command(name: impl ToString, about: impl ToString, mode: CommandMode) -> Command {
        let add = mode == CommandMode::Add;
        macro_rules! arg {
            ($name: ident, $long: literal, $value: literal, $requires: expr, $help: literal) => {{
                let mut arg = Arg::new($name.to_string())
                    .long($long)
                    .require_equals(true)
                    .value_name($value)
                    .action(ArgAction::Set)
                    .conflicts_with(TUI)
                    .help($help);
                if add {
                    for required_arg in $requires {
                        arg = arg.requires(required_arg);
                    }
                }
                arg
            }};
        }
        let mut cmd = Command::new(name.to_string())
            .about(about.to_string())
            .arg(arg!(CITY_NAME, "city", "NAME", CITY_NAME_REQS, "The location city name."))
            .arg(arg!(COUNTRY_NAME, "cn", "NAME", COUNTRY_NAME_REQS, "The location country name."))
            .arg(arg!(COUNTRY_CODE, "cc", "CODE", COUNTRY_CODE_REQS, "The location country code."))
            .arg(arg!(REGION_NAME, "rn", "NAME", REGION_NAME_REQS, "The location region name."))
            .arg(arg!(REGION_CODE, "rc", "CODE", REGION_CODE_REQS, "The location region code."))
            .arg_required_else_help(true);
        if add {
            // include the latitude and longitude if adding a location
            cmd = cmd.arg(arg!(LATITUDE, "lat", "LAT", LATITUDE_REQS, "The location latitude."));
            cmd = cmd.arg(arg!(LONGITUDE, "lng", "LNG", LONGITUDE_REQS, "The location longitude."));
        }
        cmd.arg(arg!(TZ, "tz", "TZ", TZ_REQS, "The location timezone."))
            .arg(
                Arg::new(TUI)
                    .short('t')
                    .long("tui")
                    .action(ArgAction::SetTrue)
                    .help(format!("{} the location interactively.", if add { "Add" } else { "Modify" })),
            )
            .arg(
                Arg::new(ALIAS)
                    .action(ArgAction::Set)
                    .required(true)
                    .value_name("ALIAS")
                    .help("The location alias name."),
            )
    }

    /// Used to see if the TUI option is set.
    ///
    /// # Arguments
    ///
    /// * `args` holds the command line arguments.
    ///
    pub fn is_tui(args: &ArgMatches) -> bool {
        args.get_flag(TUI)
    }

    /// Used to get the location alias name.
    ///
    /// # Arguments
    ///
    /// * `args` holds the command line arguments.
    ///
    pub fn get_alias(args: &ArgMatches) -> String {
        // the alias is always required so if this fails something is AFU
        args.get_one::<String>(ALIAS).unwrap().to_lowercase()
    }

    /// Used to get the location properties from the command line arguments.
    ///
    /// # Arguments
    ///
    /// * `args` holds the command line arguments.
    ///
    pub fn get_location(args: &ArgMatches) -> Option<Location> {
        let mut is_none = true;
        let mut get = |id: &str| -> String {
            args.get_one::<String>(id).map_or(Default::default(), |arg| {
                is_none = false;
                arg.trim().to_string()
            })
        };
        let try_get = |id: &str| -> String {
            match args.try_get_one::<String>(id) {
                // on error assume it's the missing latitude or longitude command argument
                Err(_) => String::default(),
                // on add the other arguments are required so you do not need to set is_none
                Ok(arg_opt) => arg_opt.map_or(String::default(), |arg| arg.trim().to_string()),
            }
        };
        let alias = get_alias(args);
        let city_name = get(CITY_NAME);
        let country_name = get(COUNTRY_NAME);
        let country_code = get(COUNTRY_CODE);
        let region_name = get(REGION_NAME);
        let region_code = get(REGION_CODE);
        let latitude = try_get(LATITUDE);
        let longitude = try_get(LONGITUDE);
        let tz = get(TZ);
        match is_none {
            true => None,
            false => Some(Location {
                country_name,
                country_code,
                region_name,
                region_code,
                city_name,
                alias,
                latitude,
                longitude,
                tz,
            }),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn modify_cmd() {
            // the boilerplate code to parse the command line argument
            static ALIAS: &str = "alias";
            let mut testcase = command("mt", "modify", CommandMode::Modify).arg_required_else_help(false);

            macro_rules! try_parse {
                ($args: expr) => {{
                    // create the command line from the args passed in
                    let mut cmdline = vec!["mt"];
                    cmdline.append(&mut $args);
                    cmdline.push(ALIAS);
                    testcase.try_get_matches_from_mut(&cmdline)
                }};
            }
            macro_rules! parse {
                ($args: expr) => {
                    try_parse!($args).unwrap()
                };
            }

            // no args
            let args = parse!(vec![]);
            assert!(!is_tui(&args));
            assert!(get_location(&args).is_none());

            // tui
            let args = parse!(vec!["--tui"]);
            assert!(is_tui(&args));
            assert!(get_location(&args).is_none());

            // using latitude and longitude should be an error
            assert!(try_parse!(vec!["--lat=1"]).is_err());
            assert!(try_parse!(vec!["--lng=1"]).is_err());

            macro_rules! testcase {
                ($attr: ident, $value: literal, $option: literal) => {{
                    // the expected location
                    let lhs = Location { $attr: $value.to_string(), alias: ALIAS.to_string(), ..Default::default() };
                    // parse the command line and get the location
                    let args = parse!(vec![$option]);
                    let rhs = get_location(&args).unwrap();
                    // make sure the two match
                    assert_eq!(lhs.country_name, rhs.country_name);
                    assert_eq!(lhs.country_code, rhs.country_code);
                    assert_eq!(lhs.region_name, rhs.region_name);
                    assert_eq!(lhs.region_code, rhs.region_code);
                    assert_eq!(lhs.city_name, rhs.city_name);
                    assert_eq!(lhs.alias, rhs.alias);
                    assert_eq!(lhs.latitude, rhs.latitude);
                    assert_eq!(lhs.longitude, rhs.longitude);
                    assert_eq!(lhs.tz, rhs.tz);
                }};
            }
            testcase!(country_name, "Country Name", "--cn=Country Name");
            testcase!(country_code, "Country Code", "--cc=Country Code");
            testcase!(region_name, "Region Name", "--rn=Region Name");
            testcase!(region_code, "Region Code", "--rc=Region Code");
            testcase!(city_name, "City Name", "--city=City Name");
            testcase!(tz, "UTC", "--tz=UTC");
        }
    }
}

pub mod tui {
    //! An inline terminal UI to add or modify location properties.

    use crate::cli;
    use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    use ratatui::{
        backend::Backend,
        layout::Alignment,
        prelude::{Frame, Position, Rect, Size, Stylize, Widget},
        text::Line,
        widgets::Clear,
        Terminal,
    };
    use std::ops::ControlFlow;
    use tui_lib::{
        beep, CommandBar, CommandBarStyles, Editor, EditorGroup, EditorGroupResult, EditorGroupStyles, EditorStyles,
        Label,
    };
    use weather_lib::prelude::Location;

    /// Controls the behavior of the [Editor] when it runs.
    ///
    #[derive(Eq, PartialEq, Debug)]
    pub enum LocationEditorMode {
        /// Add a new location using the alias name provided in the variant.
        New(String),
        /// Add a new location using the city properties provided in the variant.
        Add(Location),
        /// Modify properties of the location provided in the variant.
        Modify(Location),
    }

    /// The ID and label of the alias editor.
    const ALIAS: &str = "Alias: ";

    /// The ID and label of the country name editor.
    const COUNTRY_NAME: &str = "Country Name: ";

    /// The ID and label of the country code editor.
    const COUNTRY_CODE: &str = "Country Code: ";

    /// The ID and label of the region name editor.
    const REGION_NAME: &str = "Region Name: ";

    /// The ID and label of the region code editor.
    const REGION_CODE: &str = "Region Code: ";

    /// The ID and label of the city name editor.
    const CITY_NAME: &str = "City Name: ";

    /// The ID and label of the latitude editor.
    const LATITUDE: &str = "Latitude: ";

    /// The ID and label of the longitude editor.
    const LONGITUDE: &str = "Longitude: ";

    /// The ID and label of the timezone editor.
    const TIMEZONE: &str = "Timezone: ";

    /// Get an iterator of lowercase and uppercase ASCII characters.
    ///
    fn alpha() -> impl Iterator<Item = char> {
        ('a'..='z').chain('A'..='Z')
    }

    /// Get an iterator of numeric digits.
    ///
    fn digits() -> impl Iterator<Item = char> {
        '0'..='9'
    }

    /// Get an iterator of lowercase and uppercase ASCII characters including numeric digits.
    ///
    fn alphanumeric() -> impl Iterator<Item = char> {
        alpha().chain(digits())
    }

    /// The number of terminal rows needed for the inline editor.
    pub const VIEWPORT_ROWS: u16 = 14;

    /// The location editor properties.
    ///
    #[derive(Debug)]
    pub struct LocationEditor {
        /// The banner drawn above the editors.
        banner: String,
        /// The current cursor position.
        position: Option<Position>,
        /// Problems with the location properties.
        problems: Option<String>,
        /// The location property editors.
        editors: EditorGroup,
        /// The command bar renderer.
        command_bar: CommandBar,
        /// The command action keys.
        command_key: String,
        /// Used in the event loop to redraw the screen.
        redraw: bool,
    }
    impl LocationEditor {
        /// Create a new instance of the editor.
        ///
        /// # Arguments
        ///
        /// * `add_mode` will be true when the editor is being used to add a new location.
        ///
        pub fn new(mode: LocationEditorMode) -> Self {
            let (new, add, modify) = match mode {
                LocationEditorMode::New(_) => (true, false, false),
                LocationEditorMode::Add(_) => (false, true, false),
                LocationEditorMode::Modify(_) => (false, false, true),
            };
            let location = match mode {
                LocationEditorMode::New(alias) => Location { alias, ..Location::default() },
                LocationEditorMode::Add(location) => location,
                LocationEditorMode::Modify(location) => location,
            };
            macro_rules! editor {
                ($prop: ident, $id: expr, $width: expr, $readonly: expr) => {{
                    let len = location.$prop.chars().count() as u16;
                    Editor::new(location.$prop)
                        .with_id($id)
                        .with_width(if $readonly { len } else { $width })
                        .with_readonly($readonly)
                }};
            }
            let ll_width = "-###.########".len() as u16;
            let editors = vec![
                // the alias will always be active when adding
                editor!(alias, ALIAS, 30, !add)
                    .with_lowercase()
                    .with_valid_chars(alphanumeric().chain("_".chars()))
                    .with_label(Label::new(ALIAS).with_selector(if add { 'A' } else { '\0' })),
                // the city name will always be active if not adding
                editor!(city_name, CITY_NAME, 50, add)
                    .with_valid_chars(alphanumeric().chain("_-., ".chars()))
                    .with_label(Label::new(CITY_NAME).with_selector(if !add { 'N' } else { '\0' })),
                editor!(latitude, LATITUDE, ll_width, add | modify)
                    .with_valid_chars(digits().chain("-.".chars()))
                    .with_label(Label::new(LATITUDE).with_selector(if new { 't' } else { '\0' })),
                editor!(longitude, LONGITUDE, ll_width, add | modify)
                    .with_valid_chars(digits().chain("-.".chars()))
                    .with_label(Label::new(LONGITUDE).with_selector(if new { 'g' } else { '\0' })),
                editor!(tz, TIMEZONE, 30, add)
                    .with_valid_chars(alpha().chain("/_".chars()))
                    .with_label(Label::new(TIMEZONE).with_selector(if !add { 'T' } else { '\0' })),
                editor!(region_name, REGION_NAME, 30, add)
                    .with_valid_chars(alphanumeric().chain("_-. ".chars()))
                    .with_label(Label::new(REGION_NAME).with_selector(if !add { 'N' } else { '\0' })),
                editor!(region_code, REGION_CODE, 3, add)
                    .with_valid_chars(alphanumeric())
                    .with_label(Label::new(REGION_CODE).with_selector(if !add { 'C' } else { '\0' })),
                editor!(country_name, COUNTRY_NAME, 20, add)
                    .with_valid_chars(alphanumeric().chain("_-. ".chars()))
                    .with_label(Label::new(COUNTRY_NAME).with_selector(if !add { 'N' } else { '\0' })),
                editor!(country_code, COUNTRY_CODE, 3, add)
                    .with_valid_chars(alphanumeric())
                    .with_label(Label::new(COUNTRY_CODE).with_selector(if !add { 'C' } else { '\0' })),
            ];
            let mut command_bar =
                CommandBar::default().with_alignment(Alignment::Center).with_styles(CommandBarStyles::common());
            let (banner, command_key) = if new {
                command_bar = command_bar.add_command("^A", " to add location");
                (" Add Location ", "Aa".to_string())
            } else if add {
                command_bar = command_bar.add_command("^A", " to add city");
                (" Add City ", "Aa".to_string())
            } else if modify {
                command_bar = command_bar.add_command("^U", " to update location");
                (" Update Location ", "Uu".to_string())
            } else {
                unreachable!("LocationEditorMode flag new, add, or modify needs to be set...")
            };
            command_bar = command_bar.add_command("ESC", " to cancel");
            Self {
                banner: banner.to_string(),
                position: None,
                problems: None,
                command_bar,
                command_key,
                editors: EditorGroup::new(editors).with_label_alignment(Alignment::Right).with_wrap().with_styles(
                    EditorGroupStyles { active: EditorStyles::active(), inactive: EditorStyles::inactive() },
                ),
                redraw: false,
            }
        }

        /// Manage the terminal screen drawing the editor contents and reading keystrokes.
        ///
        /// # Arguments
        ///
        /// * `terminal` is used to draw contents editor contents on the screen.
        ///
        pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> cli::Result<Option<Location>> {
            let mut location_opt = None;
            self.redraw = true;
            loop {
                if self.redraw {
                    // draw the editor
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

                // get the next keystroke
                match event::read() {
                    Err(error) => cli::err!("There was a problem reading the keyboard: {error}")?,
                    Ok(Event::Mouse(mouse_event)) => self.mouse_event(mouse_event),
                    Ok(Event::Key(key_event)) => match self.key_event(key_event) {
                        ControlFlow::Break(None) => break,
                        ControlFlow::Break(Some(location)) => {
                            location_opt.replace(location);
                            break;
                        }
                        _ => (),
                    },
                    _ => (),
                }
            }
            Ok(location_opt)
        }

        /// Manage key events for the editor.
        ///
        /// # Arguments
        ///
        /// * `event` is the key event that will be processed.
        ///
        fn key_event(&mut self, event: KeyEvent) -> ControlFlow<Option<Location>> {
            // only deal with a key press
            if event.is_press() {
                match (event.modifiers, event.code) {
                    // quit the editor
                    (KeyModifiers::NONE, KeyCode::Esc) => ControlFlow::Break(None)?,
                    // check to see if the user is done with the editor
                    (KeyModifiers::CONTROL, KeyCode::Char(key)) => match self.command_key.contains(key) {
                        false => beep(),
                        true => {
                            if self.editors_ok() {
                                macro_rules! get {
                                    ($attr: expr) => {
                                        self.editors.editor($attr).unwrap().text().trim().to_owned()
                                    };
                                }
                                let location_opt = Some(Location {
                                    country_name: get!(COUNTRY_NAME),
                                    country_code: get!(COUNTRY_CODE),
                                    region_name: get!(REGION_NAME),
                                    region_code: get!(REGION_CODE),
                                    city_name: get!(CITY_NAME),
                                    alias: get!(ALIAS),
                                    latitude: get!(LATITUDE),
                                    longitude: get!(LONGITUDE),
                                    tz: get!(TIMEZONE),
                                });
                                ControlFlow::Break(location_opt)?;
                            }
                        }
                    },
                    _ => match self.editors.key_pressed(event) {
                        EditorGroupResult::Consumed => self.redraw = true,
                        _ => beep(),
                    },
                }
            }
            ControlFlow::Continue(())
        }

        /// Manage mouse events for the editor.
        ///
        /// # Arguments
        ///
        /// * `event` is the mouse event.
        ///
        fn mouse_event(&mut self, event: MouseEvent) {
            // only deal with left button mouse clicks
            if event.kind == MouseEventKind::Down(MouseButton::Left) {
                if self.editors.left_mouse_button(event) == EditorGroupResult::Consumed {
                    self.redraw = true;
                }
            }
        }

        pub fn set_problems(&mut self, problems: impl ToString) {
            self.problems.replace(problems.to_string());
        }

        /// Draw the contents of the editor onto the terminal screen.
        ///
        /// # Arguments
        ///
        /// * `frame` is the current screen frame that will be used.
        ///
        fn draw(&mut self, frame: &mut Frame) {
            let area = frame.area();
            let buffer = frame.buffer_mut();

            // create the area where the editor will be drawn
            let editors_size = self.editors.size();
            let mut view_area = Rect {
                x: area.x,
                y: area.y,
                width: std::cmp::min(area.width, std::cmp::max(editors_size.width, self.command_bar.width())),
                height: 1,
            };

            // add the banner
            Line::from(self.banner.as_str()).centered().reversed().render(view_area, buffer);
            view_area.y += 2;

            // draw the editors
            let editors_area = view_area.resize(Size { width: view_area.width, height: editors_size.height });
            self.position = self.editors.render(editors_area, buffer);
            view_area.y += editors_area.height;

            // draw any problems
            Clear::default().render(view_area, buffer);
            if let Some(problem) = &self.problems {
                Line::from(problem.as_str()).centered().red().render(view_area, buffer);
            }
            view_area.y += 1;

            // draw the banner
            self.command_bar.render(view_area, buffer);
        }

        /// Make sure the editors text is not empty.
        ///
        fn editors_ok(&mut self) -> bool {
            // clear out any previous problems
            self.problems.take();

            macro_rules! validate {
                ($attr: expr, $what: literal) => {{
                    let attr = self.editors.editor($attr).unwrap().text().trim();
                    if attr.is_empty() {
                        self.problems.replace(format!("The {} cannot be empty.", $what));
                        self.redraw = true;
                        return false;
                    }
                }};
            }
            // the validation order corresponds to the editor field order
            validate!(ALIAS, "alias name");
            validate!(CITY_NAME, "city name");
            validate!(LATITUDE, "latitude");
            validate!(LONGITUDE, "longitude");
            validate!(TIMEZONE, "timezone");
            validate!(REGION_NAME, "region name");
            validate!(REGION_CODE, "region code");
            validate!(COUNTRY_NAME, "country name");
            validate!(COUNTRY_CODE, "country code");
            true
        }
    }
}
