//! A specialized text editor for date fields.
//!
//! The current date format accepted right now is `MM/DD/YYYY` where
//!
//! - `MM` is the month (01 - 12).
//! - `DD` is the 2 digit day (01, 10, 20, etc.)
//! - `YYYY` is the 4 digit year (1970)
//!
//! The editor currently does not validate the text box contents.
//!
use super::*;

use chrono::NaiveDate;
use toolslib::date_time::{fmt_date, parse_date};

/// The specialized date text field editor.
#[derive(Debug)]
pub struct DateEditor {
    /// The date string text.
    text: String,
    /// The current position in the text field.
    column: usize,
}
impl Default for DateEditor {
    /// Create a new instance of the date editor using the `mm/dd/yyyy` text format.
    ///
    fn default() -> Self {
        Self { text: "  /  /    ".to_string(), column: 0 }
    }
}
impl DateEditor {
    /// Initialize the editor from a date.
    ///
    /// # Arguments
    ///
    /// - `date` is the initial value of the date text editor.
    ///
    pub fn with_date(mut self, date: NaiveDate) -> Self {
        self.text = Self::format(date);
        self
    }
    /// Convert the date to the correct date field format.
    ///
    /// # Arguments
    ///
    /// - `date` will be converted to the correct date field format.
    ///
    fn format(date: NaiveDate) -> String {
        fmt_date(&date, "%m/%d/%Y")
    }
    /// Move the current position to the beginning of the date field.
    ///
    fn move_front(&mut self) -> ControlFlow<ControlResult> {
        self.column = 0;
        break_event!(ControlResult::Continue)
    }
    /// Move the current position to the end of the date field.
    ///
    fn move_end(&mut self) -> ControlFlow<ControlResult> {
        self.column = self.text.len() - 1;
        break_event!(ControlResult::Continue)
    }
    /// Move the current position in the date field left. [ControlResult::NotAllowed] will be returned
    /// if the position is already at the left most position.
    ///
    fn move_left(&mut self) -> ControlFlow<ControlResult> {
        match self.column == 0 {
            true => break_event!(ControlResult::NotAllowed),
            false => {
                self.column -= 1;
                if self.on_separator() {
                    self.column -= 1;
                }
                break_event!(ControlResult::Continue)
            }
        }
    }
    /// Move the current position in the date field right. [ControlResult::NotAllowed] will be returned
    /// if the position is already at the right most position.
    ///
    fn move_right(&mut self) -> ControlFlow<ControlResult> {
        match self.column == self.text.len().saturating_sub(1) {
            true => break_event!(ControlResult::NotAllowed),
            false => {
                self.column += 1;
                if self.on_separator() {
                    self.column += 1;
                }
                break_event!(ControlResult::Continue)
            }
        }
    }
    /// Add or replace a character at the current position in the date field.
    ///
    /// # Arguments
    ///
    /// - `ch` is the character that will be added or replaced.
    ///
    fn add_char(&mut self, ch: char) -> ControlFlow<ControlResult> {
        // need to come up with a date editor to replace just adding the character
        match ch.is_digit(10) {
            false => break_event!(ControlResult::NotAllowed),
            true => {
                let mut chars = self.text.chars().collect::<Vec<char>>();
                chars[self.column] = ch;
                self.text = chars.into_iter().collect();
                // ignore the result of moving right
                let _ = self.move_right();
                break_event!(ControlResult::Continue)
            }
        }
    }
    /// Queries if the current position is on one of the date field separators.
    ///
    fn on_separator(&self) -> bool {
        self.column == 2 || self.column == 5
    }
}
impl FieldEditor for DateEditor {
    /// Get the date editor screen size.
    fn size(&self) -> Size {
        Size { width: self.text.len() as u16, height: 1 }
    }
    /// Dispatch a key pressed event to the field editor and return the result.
    ///
    /// # Arguments
    ///
    /// * `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!("DateEditor");
        match (key_event.modifiers, key_event.code) {
            (KeyModifiers::CONTROL, KeyCode::Left) => self.move_front()?,
            (KeyModifiers::CONTROL, KeyCode::Right) => self.move_end()?,
            (KeyModifiers::NONE, KeyCode::Home) => self.move_front()?,
            (KeyModifiers::NONE, KeyCode::End) => self.move_end()?,
            (KeyModifiers::NONE, KeyCode::Left) => self.move_left()?,
            (KeyModifiers::NONE, KeyCode::Right) => self.move_right()?,
            (KeyModifiers::NONE, KeyCode::Char(ch)) => self.add_char(ch)?,
            _ => (),
        }
        ControlFlow::Continue(())
    }
    /// Render the date field editor and return the current cursor coordinate.
    ///
    /// # Arguments
    ///
    /// * `area` is where on the screen the date field will be rendered.
    /// * `buffer` is where the rendering is sent.
    /// * `styles` is catalog that will be used to render the date field.
    ///
    fn render(&self, mut area: Rect, buffer: &mut Buffer, styles: &StyleCatalog) -> Position {
        log_render!("DateEditor");
        area.width = cmp::min(area.width, self.text.len() as u16);
        Paragraph::new(self.text.as_str()).style(styles.get(StyleId::Text)).render(area, buffer);
        Position::new(area.x + self.column as u16, area.y)
    }
    /// Return the date field text.
    fn text(&self) -> &str {
        &self.text
    }
    /// Set the field text.
    ///
    /// # Arguments
    ///
    /// * `content` is the date field text.
    ///
    fn set_text(&mut self, text: impl ToString) {
        let text = text.to_string();
        match parse_date(&text) {
            Ok(date) => self.text = Self::format(date),
            Err(err) => {
                log::error!("Error creating date from '{}' ({}).", text, err);
                self.text = "01/01/1970".to_string();
            }
        }
        self.column = 0;
    }
}
