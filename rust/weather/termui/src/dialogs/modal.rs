//! The various modal dialog implementations.

use super::*;
pub use button::ButtonDialog;
pub use menu::MenuDialog;
pub use message::{MessageDialog, MessageStyle};
pub use progress::ProgressDialog;

mod button;
mod menu;
mod message;
mod progress;

/// A frame manages the screen area used by a dialog.
#[derive(Debug, Default)]
struct DialogFrame {
    /// Most dialogs will show a message so provide a common location.
    message: Option<MessageDialog>,
    /// Controls if a dialog will have a border surrounding it.
    draw_border: bool,
}
impl DialogFrame {
    /// A builder method to include a border around the dialog when rendered.
    ///
    fn with_border(mut self) -> Self {
        self.draw_border = true;
        self
    }
    /// Dispatch a key pressed event to the frame. If there is a message, events will be passed on to
    /// it until dismissed. [ControlFlow::Continue] will be returned if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("DialogFrame");
        match self.message.take() {
            Some(mut message) => match message.key_pressed(key_event) {
                ControlFlow::Break(DialogResult::Exit) => break_event!(DialogResult::Continue),
                result => {
                    self.message.replace(message);
                    result
                }
            },
            None => match (key_event.modifiers, key_event.code) {
                (KeyModifiers::NONE, KeyCode::Esc) => {
                    break_event!(DialogResult::Cancel)
                }
                _ => ControlFlow::Continue(()),
            },
        }
    }
    /// Draw the dialog frame on the terminal screen. ***Note:*** This does not include drawing the
    /// message dialog, the dialog or window need to do that.
    ///
    /// # Arguments
    ///
    /// - `title` is a description of the frame contents.
    /// - `area` is where on the terminal the frame will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` contains the [styles](StyleCatalog) used the draw the frame.
    ///
    fn render(&self, title: Option<&String>, area: Rect, buffer: &mut Buffer, styles: &StyleCatalog) {
        log_render!("DialogFrame");
        Clear::default().render(area, buffer);
        let title = title.as_ref().map_or("", |title| title.as_str());
        let frame_borders = if self.draw_border { Borders::ALL } else { Borders::NONE };
        Block::default()
            .borders(frame_borders)
            .title(Line::styled(title, styles.get(StyleId::DialogTitle)))
            .title_alignment(Alignment::Center)
            .border_style(styles.get(StyleId::DialogBorder))
            .style(styles.get(StyleId::Screen))
            .render(area, buffer);
    }
}
