//! The TUI button dialog.
//!
//! This modal dialog manages a [DialogWindow] and a [ButtonBar]. The `DialogWindow` is drawn at the
/// top portion of the screen area and the `ButtonBar` below it.
///
use super::*;
use controls::ButtonBar;
use std::fmt::Debug;

/// The button dialog.
///
#[derive(Debug)]
pub struct ButtonDialog<T: DialogWindow + Debug> {
    /// The dialog title.
    title: Option<String>,
    /// The frame of the dialog.
    frame: DialogFrame,
    /// The dialog buttons.
    buttons: ButtonBar,
    /// The window managed by the dialog.
    window: T,
    /// The button dialog style catalog type. This will always be [CatalogType::ButtonDialog].
    pub catalog_type: CatalogType,
}
impl<T: DialogWindow + Debug> ButtonDialog<T> {
    /// Create a new instance of the dialog.
    ///
    /// # Arguments
    ///
    /// - `buttons` is the collection of dialog buttons.
    /// - `window` is the window that will be managed.
    ///
    pub fn new(buttons: ButtonBar, window: T) -> Self {
        let frame = DialogFrame::default().with_border();
        Self { title: None, frame, buttons, window, catalog_type: CatalogType::ButtonDialog }
    }
    /// A builder method that sets the dialog title.
    ///
    /// # Arguments
    ///
    /// - `title` is the dialog description.
    ///
    pub fn with_title(mut self, title: impl ToString) -> Self {
        self.title = Some(title.to_string());
        self
    }
    /// Get the width of the dialog description.
    ///
    pub fn title_width(&self) -> u16 {
        self.title.as_ref().map_or(0, |title| title.len() as u16)
    }
    /// Get a reference to the managed window.
    ///
    pub fn win(&self) -> &T {
        &self.window
    }
    /// Get a mutable reference to the managed window.
    ///
    pub fn win_mut(&mut self) -> &mut T {
        &mut self.window
    }
    /// Consume a key pressed event. The event will be passed to the frame if there is
    /// a message dialog otherwise it is passed on to the window and then to the frame.
    /// [ControlFlow::Continue] will be returned if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    pub fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("ButtonDialog");
        match self.frame.message.is_some() {
            true => self.frame.key_pressed(key_event)?,
            false => {
                self.window.key_pressed(key_event)?;
                if let ControlFlow::Break(ControlResult::Selected(id)) = self.buttons.key_pressed(key_event) {
                    break_event!(DialogResult::Selected(id))?;
                }
                self.frame.key_pressed(key_event)?;
            }
        }
        beep();
        ControlFlow::Continue(())
    }
    /// Draw the dialog on the terminal screen. The order of rendering is the frame, the buttons,
    /// the window, and if set the message dialog. The screen position of the cursor will be returned
    /// if available.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the dialog can be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    pub fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("ButtonDialog");
        let styles = self.catalog_type.get_styles(ControlState::Active);
        let (frame_area, window_area, buttons_area) = self.dialog_areas(area);
        // render the frame and get the area available for rendering
        self.frame.render(self.title.as_ref(), frame_area, buffer, styles);
        let mut coord = match self.buttons.render(buttons_area, buffer, styles) {
            None => None,
            Some(buttons_coord) => Some(buttons_coord),
        };
        if let Some(window_coord) = self.window.render(window_area, buffer) {
            coord.replace(window_coord);
        }
        if let Some(message) = self.frame.message.as_ref() {
            if let Some(message_coord) = message.render(area, buffer) {
                coord.replace(message_coord);
            }
        }
        coord
    }
    /// Create a dialog message.
    ///
    /// # Arguments
    ///
    /// - `style` determines how the message dialog will be drawn.
    /// - `message` is the text that will be displayed.
    ///
    pub fn set_message(&mut self, style: MessageStyle, message: impl ToString) {
        self.frame.message.replace(MessageDialog::new(style, message));
    }
    /// Create the drawing areas for the frame, window, and buttons. The dialog is
    /// centered within the area.
    ///
    /// # Arguments
    ///
    /// - `area` is where the dialog will be drawn.
    ///
    fn dialog_areas(&self, area: Rect) -> (Rect, Rect, Rect) {
        let window_size = self.window.size();
        let buttons_size = self.buttons.size();
        let mut width = if window_size.width == 0 { area.width } else { window_size.width };
        let mut height = if window_size.height == 0 { area.height } else { window_size.height };
        // include the separator and buttons height
        height += buttons_size.height + 1;
        // add in the dialog borders (bottom does not have a separator row)
        width += 4;
        height += 3;
        let frame_area = center_rect!(area, [cmp::max(width, buttons_size.width), height]);
        // the height needs to take into account the borders and inner margin
        let inner_area = inner_rect(frame_area, (2, 2), (-2, -1));
        // the client height depends on the inner_area
        let window_height = if inner_area.height < (window_size.height + buttons_size.height + 1) {
            inner_area.height.saturating_sub(buttons_size.height + 1)
        } else {
            window_size.height
        };
        let window_area = inner_rect(inner_area, (0, 0), (0, window_height as i32));
        let buttons_area = inner_rect(inner_area, (1, i32::from(window_height + 1)), (0, 0));
        (frame_area, window_area, buttons_area)
    }
}
