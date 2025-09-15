//! A terminal UI button control.
//!
//! The [Button] is used to signal some type of action should be taken. A `Button`
//! always has an identifier, label, and selector key. The button identifier is
//! returned from a `key pressed` event when the button is selected.
//!
//! The `Button` can be selected using an `ALT-key` sequence or `key` press where `key`
//! matches the selector character. It can also be configured to select when active
//! and the `Enter` key is pressed.
//!
//! A [ButtonBar] control is available to manage a horizontal group of `Button` controls.
//!
use super::*;
use styles::{StyleCatalog, StyleId};

/// The [ok](ok_button) control identifier.
///
pub const OK_BUTTON_ID: &'static str = "OK";

/// The [cancel](cancel_button) control identifier.
///
pub const CANCEL_BUTTON_ID: &'static str = "CANCEL";

/// The button data structure.
///
#[derive(Debug)]
pub struct Button {
    /// Use a label because it already has the required metadata definitions.
    label: Label,
}
impl std::fmt::Display for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Button[{}] active={}", self.label.id(), self.label.is_active())
    }
}
impl Button {
    /// Creates a button instance from the supplied metadata (it is not active by default).
    ///
    /// # Arguments
    ///
    /// - `id` is the control identifier.
    /// - `text` is the button description.
    /// - `selector` allows a button to be 'clicked' when a key is pressed.
    ///
    pub fn new(id: impl ToString, text: impl ToString, selector: char) -> Self {
        Self { label: Label::align_center(text).with_id(id).with_selector(selector) }
    }
    /// A builder method that will set the button into an active state.
    ///
    pub fn with_active(mut self) -> Self {
        self.label.set_active(true);
        self
    }
}
/// The control API implementation.
impl Control for Button {
    /// Get the button identifier attribute.
    fn id(&self) -> &str {
        self.label.id()
    }
    /// Get the button selection character attribute.
    fn selector(&self) -> char {
        self.label.selector()
    }
    /// Get the size of the button.
    fn size(&self) -> Size {
        let mut size = self.label.size();
        size.width += 2;
        size.height += 2;
        size
    }
    /// Find out if the button is active or not.
    fn is_active(&self) -> bool {
        self.label.is_active()
    }
    /// Set the buttons active state.
    ///
    /// # Arguments
    ///
    /// - `active` is the buttons active state.
    ///
    fn set_active(&mut self, active: bool) {
        self.label.set_active(active);
    }
    /// Draw the button on the terminal screen.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the button will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` contains the [styles](StyleCatalog) used the draw the button.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer, catalog: &StyleCatalog) -> Option<Position> {
        log_render!(self.to_string());
        if area.height == 0 {
            None
        } else {
            let borders = match area.height {
                1 => Borders::TOP,
                2 => Borders::TOP | Borders::LEFT | Borders::RIGHT,
                _ => Borders::ALL,
            };
            Block::default().borders(borders).border_style(catalog.get(StyleId::ButtonBorder)).render(area, buffer);
            let label_area = area.inner(Margin { horizontal: 1, vertical: 1 });
            self.label.render(label_area, buffer, &catalog)
        }
    }
    /// Consume a key pressed event. The button will return [ControlFlow::Continue] if the event was not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!(self.to_string());
        self.label.key_pressed(key_event)
    }
}

/// A managed collection of [Button] controls.
///
#[derive(Debug)]
pub struct ButtonBar {
    /// The collection of [button](Button) controls.
    buttons: Vec<Button>,
    /// When set `true` a button in the collection can be selected by pressing its selector character.
    auto_select: bool,
    /// The maximum width of a controls in the collection.
    max_button_width: u16,
    /// The size (width and height) of the button collection.
    size: Size,
}
impl ButtonBar {
    /// The number of spaces between buttons (1 column left, 1 column right).
    ///
    const SEPARATOR_WIDTH: u16 = 2;
    /// Create the collection of buttons (auto select will be false by default).
    ///
    /// # Arguments
    ///
    /// - `buttons` is the collection that will be managed.
    ///
    pub fn new(buttons: Vec<Button>) -> Self {
        let mut height = 0u16;
        let mut max_button_width = 0u16;
        buttons.iter().for_each(|button| {
            let size = button.size();
            height = cmp::max(height, size.height);
            max_button_width = cmp::max(max_button_width, size.width)
        });
        let buttons_len = buttons.len() as u16;
        let width = (buttons_len * max_button_width) + (buttons_len.saturating_sub(1) * Self::SEPARATOR_WIDTH);
        let size = Size { width, height };
        Self { buttons, auto_select: false, max_button_width, size }
    }
    /// Enables or disables selection of a button by its selector.
    ///
    /// # Arguments
    ///
    /// -`yes_no` is the state of auto selection for the collection.
    ///
    pub fn with_auto_select(mut self, yes_no: bool) -> Self {
        self.auto_select = yes_no;
        self
    }
    /// Get the size of the button collection.
    pub fn size(&self) -> Size {
        self.size
    }
    /// Handle a key event for the collection.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a key pressed terminal event.
    ///
    pub fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!("ButtonBar");
        match self.auto_select && key_event.modifiers == KeyModifiers::NONE && key_event.code == KeyCode::Enter {
            true => {
                if let Some(button) = self.buttons.iter().find(|button| button.is_active()) {
                    return ControlFlow::Break(ControlResult::Selected(button.id().to_string()));
                }
            }
            false => {
                for button in &mut self.buttons {
                    button.label.key_pressed(&key_event)?;
                }
            }
        }
        ControlFlow::Continue(())
    }
    /// Show the collection of buttons on the terminal screen. The buttons area drawn on the screen from left to right.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the button will be shown.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` contains the [styles](StyleCatalog) used the draw the button.
    ///
    pub fn render(&self, area: Rect, buffer: &mut Buffer, catalog: &StyleCatalog) -> Option<Position> {
        log_render!("ButtonBar");
        let buttons_area = center_rect!(area, [self.size.width, self.size.height]);
        let mut button_area = inner_rect(buttons_area, (0, 0), (self.max_button_width as i32, 0));
        let mut active_coordinates = None;
        for button in &self.buttons {
            if button_area.right() > area.right() {
                break;
            }
            if let Some(coordinates) = button.render(button_area, buffer, catalog) {
                active_coordinates.replace(coordinates);
            }
            button_area.x += self.max_button_width + Self::SEPARATOR_WIDTH;
        }
        active_coordinates
    }
}

/// Create a 'Ok' button with [OK_BUTTON_ID] as the control identifier.
pub fn ok_button() -> Button {
    Button::new(OK_BUTTON_ID, " Ok ", 'O')
}

/// Create a 'Cancel' button with [CANCEL_BUTTON_ID] as the control identifier.
pub fn cancel_button() -> Button {
    Button::new(CANCEL_BUTTON_ID, "Cancel", 'C')
}
