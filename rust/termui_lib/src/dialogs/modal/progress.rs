//! The TUI progress dialog.

use super::*;
use std::{cell::RefCell, time::{Duration, SystemTime}};

/// A progress indicator that bounces back and forth.
///
#[derive(Debug)]
struct CylonEye {
    /// The characters that make up the progress indicator.
    eye: String,
    /// The current position of the eye.
    position: usize,
    /// The width of the field the eye moves in.
    width: usize,
    /// The current direction is to the right.
    move_right: bool,
}
impl Default for CylonEye {
    /// Creates a new [CylonEye].
    fn default() -> Self {
        Self {
            eye: symbols::line::THICK_HORIZONTAL.repeat(Self::EYE_WIDTH),
            position: 0,
            width: 20,
            move_right: true,
        }
    }
}
impl CylonEye {
    /// The number of character in the eye.
    const EYE_WIDTH: usize = 5;
    /// Get the next progress indicator.
    fn next(&mut self) -> String {
        let indicator = format!("{:offset$}{}", " ", self.eye, offset = self.position);
        match self.move_right {
            true => match self.position + 1 + Self::EYE_WIDTH < self.width {
                true => self.position += 1,
                false => {
                    self.move_right = false;
                    self.position -= 1;
                }
            }
            false => match self.position != 0 {
                true => self.position -= 1,
                false => {
                    self.move_right = true;
                    self.position += 1;
                }
            }
        }
        indicator
    }
}

/// A dialog that shows a message and progress indicator.
///
#[derive(Debug)]
pub struct ProgressDialog {
    /// The frame of the dialog.
    frame: DialogFrame,
    /// The description of what's being tracked.
    description: String,
    /// The size of the dialog.
    size: Size,
    /// The next render time.
    next_render: RefCell<SystemTime>,
    /// The milliseconds to wait before the next rendering.
    render_duration: u64,
    /// The progress indicator,
    progress_indicator: RefCell<CylonEye>,
    /// The button dialog style catalog type. This will always be [CatalogType::ProgressDialog].
    pub catalog_type: CatalogType,
}
impl ProgressDialog {
    /// Create a new progress dialog.
    ///
    /// # Arguments
    ///
    /// - `description` describes what progress is being made.
    ///
    pub fn new(description: impl ToString) -> Self {
        let description = description.to_string();
        let cylon_eye = CylonEye::default();
        // there is a border and space surrounding the description
        let width = cmp::max(description.len(), cylon_eye.width) as u16 + 4;
        Self {
            frame: DialogFrame::default().with_border(),
            description: description.to_string(),
            size: Size { width, height: 6 },
            next_render: RefCell::new(SystemTime::now()),
            render_duration: 20,
            progress_indicator: RefCell::new(cylon_eye),
            catalog_type: CatalogType::ProgressDialog,
        }
    }
    /// Consume a key pressed event always returning [Continue](DialogResult::Continue).
    ///
    pub fn key_pressed(&mut self, _key_event: KeyEvent) -> ControlFlow<DialogResult> {
        break_event!(DialogResult::Continue)
    }
    /// Draw the progress dialog centered on the terminal screen. The cursor screen position will always be `None`.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the dialog can be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    pub fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("ProgressDialog");
        // check if the dialog should be rendered or not
        let current_time = SystemTime::now();
        if *self.next_render.borrow() > current_time {
            None?;
        }
        let next_render = current_time.checked_add(Duration::from_millis(self.render_duration)).unwrap();
        self.next_render.replace(next_render);
        // set up the frame
        let frame_area = center(area, self.size);
        let styles = self.catalog_type.get_styles(ControlState::Active);
        self.frame.render(None, frame_area, buffer, styles);
        let content_area = inner_rect(frame_area, (2, 2), (-2, -1));
        // show the description
        let description_area = inner_rect(content_area, (0, 0), (0, 1));
        Paragraph::new(Line::raw(&self.description))
            .style(styles.get(StyleId::Text))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .render(description_area, buffer);
        // show the indicator
        let progress_indicator = self.progress_indicator.borrow_mut().next();
        let indicator_line = inner_rect(content_area, (0, -1), (0, 0));
        let indicator_area = center_rect!(indicator_line, [self.progress_indicator.borrow().width as u16, 1]);
        Paragraph::new(Line::raw(progress_indicator))
            .style(styles.get(StyleId::Highlight))
            .alignment(Alignment::Left)
            .render(indicator_area, buffer);
        None
    }
}
