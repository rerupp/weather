//! The Terminal based UI used by the weather CLI.
//!
//! This library contains [dialogs], [menus], and [controls] that sit on top of the
//! ratatui crate. Originally I simply wanted to front end list locations
//! allowing locations and weather history to be added. That kind of morphed
//! into a collection of widgets that can be used for other things.
//!
//! All [dialogs], [menus], and [controls] implement 2 methods.
//!
//! > `fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {...}`
//! >
//! > This method is guaranteed to receive a `KeyEventKind::Press` key event.
//! > `ControlFlow::Continue(())` will be returned if the component does not consume
//! > the event.
//!
//! > `fn render(&self, area: Rect, buffer: &mut Buffer, styles: &StyleCatalog) -> Option<Position> {...}`
//! >
//! > This method is used to draw a component on the terminal screen. The component will
//! > return the current cursor position if appropriate.
//!
//! All components are drawn on the screen using ratatui styles. The styles are contained in
//! a [StyleCatalog](styles::StyleCatalog) specific to a [CatalogType](styles::CatalogType).
//!
//! There are some silly things going on in the library where ratatui components
//! are instanced instead of implementing the ratatui widget. At some point I
//! might do that once the library is moved peer to the toolslib library.
//!
//!
#![allow(rustdoc::private_intra_doc_links)]

use std::{
    cmp,
    io::{self, stdout, Write},
    ops::ControlFlow,
};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Position, Rect, Size},
    prelude::*,
    widgets::*,
};

/// The [ControlFlow::Break] variant is used all over the place so create a shorthand for it.
///
/// Using `break_event!(ControlResult::Continue)` results in `ControlFlow::Break(ControlResult::Continue)`.
///
#[macro_export]
macro_rules! break_event {
    ($control_result:expr) => {
        ControlFlow::Break($control_result)
    };
}

/// Log messages and elapsed time for methods.
///
/// The class logs `DEBUG` messages when a new instance is created and when it goes out of
/// scope. The first message follows the form "`what feature`" or "`what feature debug`".
/// The ending message follows the form "`what feature elapsed 0ms`".
///
pub struct LogFeatureStartStop {
    /// The type of feature (currently "render" or "key_pressed").
    feature: String,
    /// The description of what is being timed (currently the classname is used).
    what: String,
    /// The elapsed timer tracks when the class was created until when the class is dropped.
    timer: toolslib::stopwatch::StopWatch,
}
impl Drop for LogFeatureStartStop {
    /// On drop log the elapsed time.
    fn drop(&mut self) {
        log::debug!("{} {} elapsed {}", self.what, self.feature, self.timer);
    }
}
impl LogFeatureStartStop {
    /// Create the feature logger.
    ///
    /// # Arguments
    ///
    /// - `feature` is the feature description.
    /// - `what` is the function description.
    /// - `debug` is optional debug information.
    ///
    pub fn new(feature: impl ToString, what: impl ToString, debug: Option<String>) -> Self {
        let feature = feature.to_string();
        let what = what.to_string();
        match debug {
            None => log::debug!("{} {}", what, feature),
            Some(message) => log::debug!("{} {} {}", what, feature, message),
        }
        Self { what, feature, timer: toolslib::stopwatch::StopWatch::start_new() }
    }
}

/// Track TUI render events.
///
/// Normally this will be the first line of a `render(...)` method. A variable named `__log_render__`
/// is initialized with an instance of [LogFeatureStartStop]. There are two ways to use the macro.
///
/// - `log_render!("Classname")` will log the message "`Classname render`".
/// - `log_render!("Classname", "debug information")` will log the message "`Classname render debug information`".
/// - `log_render!("Classname", "{}", "debug information")` will log the message "`Classname render debug information`".
/// The arguments in this case are passed off to `format!` so the same rules apply.
///
#[macro_export]
macro_rules! log_render {
    ($what:expr, $($debug:tt)*) => {
        #[cfg(debug_assertions)]
        #[cfg(feature = "log_render")]
        let __log_render__ = LogFeatureStartStop::new("render", $what, Some(format!($($debug)*)));
    };
    ($what:expr) => {
        #[cfg(debug_assertions)]
        #[cfg(feature = "log_render")]
        let __log_render__ = LogFeatureStartStop::new("render", $what, None);
    };
}

/// Track TUI key pressed events.
///
/// Normally this will be the first line of a `key_pressed(...)` method. A variable named `__log_key_pressed__`
/// is initialized with an instance of [LogFeatureStartStop]. There are two ways to use the macro.
///
/// - `log_key_pressed!("Classname")` will log the message "`Classname key_pressed`".
/// - `log_key_pressed!("Classname", "debug information")` will log the message "`Classname key_pressed debug information`".
/// - `log_key_pressed!("Classname", "{}", "debug information")` will log the message "`Classname key_pressed debug information`".
/// The arguments in this case are passed off to `format!` so the same rules apply.
///
#[macro_export]
macro_rules! log_key_pressed {
    ($what:expr, $($debug:tt)*) => {
        #[cfg(debug_assertions)]
        #[cfg(feature = "log_key_event")]
        let __log_key_pressed__ = LogFeatureStartStop::new("key_pressed", $what, Some(format!($($debug)*)));
    };
    ($what:expr) => {
        #[cfg(debug_assertions)]
        #[cfg(feature = "log_key_event")]
        let __log_key_pressed__ = LogFeatureStartStop::new("key_pressed", $what, None);
    };
}

/// The library result.
pub type Result<T> = std::result::Result<T, Error>;

/// The library error.
#[derive(Debug)]
pub struct Error(String);
impl Error {
    pub fn new(msg: impl ToString) -> Self {
        Self(format!("tui: {}", msg.to_string()))
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<String> for Error {
    /// Create an error from the provided string.
    fn from(error: String) -> Self {
        Self::new(error)
    }
}
impl From<io::Error> for Error {
    // this assumes IO errors will only come from Ratatui
    fn from(error: io::Error) -> Self {
        Self::new(&format!("IO error occurred ({}).", error))
    }
}

/// Create a collection of references.
///
/// ```no_execute
/// let vec = vec![String::default()];
/// let references: Vec<&String> = as_refs!(vec);
/// ```
macro_rules! as_refs {
    ($collection:expr) => {
        $collection.iter().map(|t| t).collect()
    };
}
use as_refs;

/// Create a collection of mutable references.
///
/// ```no_execute
/// let mut vec = vec![String::default()];
/// let references: Vec<&mut String> = as_refs!(vec);
/// ```
macro_rules! as_mut_refs {
    ($collection:expr) => {
        $collection.iter_mut().map(|t| t).collect()
    };
}
use as_mut_refs;

mod console;

mod controls;

mod dialogs;

mod menus;

/// Styles used to draw the controls and dialogs.
mod styles;

pub mod prelude {
    //! This is the full list of things available from the library.
    pub use super::{
        beep, break_event, center,
        console::{Application, ApplicationResult, Console},
        controls::{
            cancel_button, ok_button, Button, ButtonBar, Checkbox, CheckBoxGroup, Control, ControlGroup, ControlResult,
            DateEditor, EditControl, EditField, EditFieldGroup, Editor, Label, ReportView, TextEditor,
            CANCEL_BUTTON_ID, OK_BUTTON_ID,
        },
        dialogs::{
            ButtonDialog, DialogResult, DialogWindow, MenuDialog, MessageDialog, MessageStyle, ProgressDialog,
            TabDialog, TabWindow,
        },
        inner_rect, log_key_pressed, log_render,
        menus::{MenuItem, MenuState, Menubar, PopupMenu},
        styles::{ActiveNormalStyles, CatalogType, ControlState, StyleCatalog, StyleCatalogs, StyleId},
        LogFeatureStartStop,
    };
}

/// Provide a common way to make a terminal beep.
///
pub fn beep() {
    // todo: make this a debug feature???
    // log::trace!("{}", std::backtrace::Backtrace::force_capture());
    let mut stdout = stdout();
    if let Err(err) = stdout.write(&[7]) {
        log::error!("Could not beep terminal ({}).", err);
    }
}

/// Create a [Rect] that sits within a given area.
///
/// # Arguments
///
/// * `(top_x, top_y)` is the upper left, top coordinate of the area.
/// * `(lower_right, lower_bottom)` is the right bottom coordinate of the area (the coordinate just outside the area).
///
pub fn inner_rect(area: Rect, (upper_x, upper_y): (i32, i32), (lower_right, lower_bottom): (i32, i32)) -> Rect {
    // make sure the coordinates are within bounds
    debug_assert!(upper_x.abs() < u16::MAX as i32);
    debug_assert!(upper_y.abs() < u16::MAX as i32);
    debug_assert!(lower_right.abs() < u16::MAX as i32);
    debug_assert!(lower_bottom.abs() < u16::MAX as i32);
    // eprintln!("  inner rect area {:?}\n  {:?}\n  {:?}", area, upper, lower);
    // get the area coordinates
    let area_top = area.top();
    let area_left = area.left();
    let area_right = area.right();
    let area_bottom = area.bottom();
    // get the upper left coordinates
    let upper_x = if upper_x < 0 {
        area_right.saturating_sub(upper_x.abs() as u16)
    } else {
        area_left + upper_x as u16
    }
    .clamp(area_left, area_right);
    let upper_y = if upper_y < 0 {
        area_bottom.saturating_sub(upper_y.abs() as u16)
    } else {
        area_top.saturating_add(upper_y as u16)
    }
    .clamp(area_top, area_bottom);
    // get the lower right coordinates
    let lower_right = if lower_right == 0 {
        area_right
    } else if lower_right < 0 {
        area_right.saturating_sub(lower_right.abs() as u16)
    } else {
        area_left + lower_right as u16
    }
    .clamp(upper_x, area_right);
    let lower_bottom = if lower_bottom == 0 {
        area_bottom
    } else if lower_bottom < 0 {
        area_bottom.saturating_sub(lower_bottom.abs() as u16)
    } else {
        area_top.saturating_add(lower_bottom as u16)
    }
    .clamp(upper_y, area_bottom);
    // create the resulting Rect
    let width = lower_right - upper_x;
    let height = lower_bottom - upper_y;
    Rect { x: upper_x, y: upper_y, width, height }
}

/// Center a rectangle within some area.
///
/// # Args
///
/// `area` is the outer rectangle.
/// `size` is the inner rectangle area.
///
pub fn center(area: Rect, size: Size) -> Rect {
    let width = cmp::min(area.width, size.width);
    let height = cmp::min(area.height, size.height);
    let x_offset = area.width.saturating_sub(width) / 2;
    let y_offset = area.height.saturating_sub(height) / 2;
    Rect { x: area.x + x_offset, y: area.y + y_offset, width, height }
}

/// Provide a shorthand for calling the [center] method.
///
/// ```no_execute
/// use ratatui::layout::Rect;
/// let area = Rect::new(0, 0, 80, 24);
/// assert_eq!(center_rect!(area, [40, 12]), center(area, Size{ width: 40, height: 12 }));
/// ```
#[macro_export]
macro_rules! center_rect {
    ($area:expr, [ $width:expr, $height:expr ]) => {
        $crate::prelude::center($area, Size { width: $width, height: $height })
    };
}
