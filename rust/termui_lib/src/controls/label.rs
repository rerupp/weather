//! The terminal UI label control.
//!
//! The [Label] provides a read-only text box. A `Label` always has a description.
//! Optionally the `Label` can be configured to have an identifier and selector key.
//!
//! The `Label` is a basic component of many other controls.
//!
use super::*;
use crate::styles::{StyleCatalog, StyleId};

/// OMG: Rust and a Null???
const NO_SELECTOR: char = '\0';

/// A basic text label control.
///
/// The label control is mostly used by other controls. It manages the common metadata
/// used to track the state of a control and basic interactions with the terminal.
///
#[derive(Debug)]
pub struct Label {
    /// The label identifier is optional.
    id: Option<String>,
    /// The textual description of the label.
    text: String,
    /// If a width has been set, align the description accordingly.
    alignment: Alignment,
    /// A character in the description that can select the label.
    selector: char,
    /// Indicates the label is currently active or not.
    active: bool,
    /// The description width.
    width: u16,
}
impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Label[")?;
        if let Some(id) = self.id.as_ref() {
            write!(f, "{}", id)?;
        }
        write!(f, "] active={}", self.active)
    }
}
impl Label {
    /// Used internally to set the initial state of the label metadata. The initial width is set to
    /// the length of the text description.
    ///
    /// # Arguments
    ///
    /// - `text` is the label description.
    /// - `alignment` is how text will be aligned within the label width.
    ///
    fn new(text: impl ToString, alignment: Alignment) -> Self {
        let text = text.to_string();
        let width = text.len() as u16;
        Self { id: None, text, alignment, selector: NO_SELECTOR, active: false, width }
    }
    /// Create a left aligned label control.
    ///
    /// # Arguments
    ///
    /// - `text` is the label description.
    ///
    pub fn align_left(text: impl ToString) -> Self {
        Self::new(text, Alignment::Left)
    }
    /// Create a right aligned label control.
    ///
    /// # Arguments
    ///
    /// - `text` is the label description.
    ///
    pub fn align_right(text: impl ToString) -> Self {
        Self::new(text, Alignment::Right)
    }
    /// Create a centered label control.
    ///
    /// # Arguments
    ///
    /// - `text` is the label description.
    ///
    pub fn align_center(text: impl ToString) -> Self {
        Self::new(text, Alignment::Center)
    }
    /// A builder method that explicitly sets the width of the label description.
    ///
    /// # Arguments
    ///
    /// - `width` is the label width.
    ///
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }
    /// A builder method that sets the label identifier.
    ///
    /// # Arguments
    ///
    /// - `id` is the label identifier.
    ///
    pub fn with_id(mut self, id: impl ToString) -> Self {
        self.id.replace(id.to_string());
        self
    }
    /// A builder method that sets the label selector character.
    ///
    /// # Arguments
    ///
    /// - `selector` is the label selector.
    ///
    pub fn with_selector(mut self, selector: char) -> Self {
        self.selector = selector;
        self
    }
    /// A builder method that sets the label active.
    ///
    pub fn with_active(mut self) -> Self {
        self.active = true;
        self
    }
    /// Get the label description.
    pub fn text(&self) -> &str {
        &self.text
    }
}
impl Control for Label {
    /// Get the label identifier or return an empty string if one has not been set.
    fn id(&self) -> &str {
        self.id.as_ref().map_or("", |id| id.as_str())
    }
    /// Get the label selector character.
    fn selector(&self) -> char {
        self.selector
    }
    /// Get the size of the label.
    fn size(&self) -> Size {
        Size { width: self.width, height: 1 }
    }
    /// Query the metadata to see if the label is active or not.
    fn is_active(&self) -> bool {
        self.active
    }
    /// Set the active state of the label.
    ///
    /// # Arguments
    ///
    /// - `active` indicates the label state.
    ///
    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
    /// Draw the label on the terminal screen.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the checkbox will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` contains the [styles](StyleCatalog) used the draw the checkbox.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer, catalog: &StyleCatalog) -> Option<Position> {
        log_render!(self.to_string());
        let text_style = catalog.get(StyleId::LabelText);
        let selector_style = catalog.get(StyleId::LabelSelector);
        let cell_width = area.width as usize;
        let text = match self.alignment {
            Alignment::Left => format!("{:<cell_width$}", self.text),
            Alignment::Center => format!("{:^cell_width$}", self.text),
            Alignment::Right => format!("{:>cell_width$}", self.text),
        };
        let line = if self.selector == NO_SELECTOR {
            Line::styled(&text, text_style)
        } else {
            Line::from(hotkey_spans(&text, self.selector, text_style, selector_style))
        };
        Paragraph::new(line).render(area, buffer);
        match self.active {
            true => match text.find(self.selector) {
                None => None,
                Some(offset) => Some(Position::new(area.x + offset as u16, area.y)),
            },
            false => None,
        }
    }
    /// Consume a key pressed event. The label will return [Continue](ControlFlow::Continue) if the event was
    /// not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a key pressed event.
    ///
    fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!(self.to_string());
        if self.selector != NO_SELECTOR {
            macro_rules! is_selector {
                ($char:expr) => {{
                    if $char.to_lowercase().to_string() == self.selector.to_lowercase().to_string() {
                        let id = self.id.as_ref().map_or(String::new(), |id| id.to_string());
                        break_event!(ControlResult::Selected(id))?;
                    }
                }};
            }
            match (key_event.modifiers, key_event.code) {
                (KeyModifiers::ALT, KeyCode::Char(ch)) => is_selector!(ch),
                (KeyModifiers::NONE, KeyCode::Char(ch)) => is_selector!(ch),
                _ => (),
            }
        }
        ControlFlow::Continue(())
    }
}
