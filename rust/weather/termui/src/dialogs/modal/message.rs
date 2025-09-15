//! The TUI message dialog.
//!
//! The [MessageDialog] is a specialized modal dialog that displays a message
//! with an [Ok](ok_button).

use super::*;
use controls::{ok_button, Button, Control};

/// The available message dialog styles.
#[derive(Debug)]
pub enum MessageStyle {
    /// The dialog draws an error message.
    Error,
    /// The dialog draws a warning message.
    Warning,
    /// The dialog draws an informational message.
    Normal,
}
impl std::fmt::Display for MessageStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A modal message display dialog.
/// .
#[derive(Debug)]
pub struct MessageDialog {
    /// The text that will be displayed.
    message: Vec<String>,
    /// The type of message dialog.
    message_style: MessageStyle,
    /// The ok button.
    button: Button,
    /// The size of the message dialog.
    size: Size,
    /// The menu dialog style catalog type. This will always be [CatalogType::MessageDialog].
    pub catalog_type: CatalogType,
}
impl MessageDialog {
    /// Create a new instance of the message dialog.
    ///
    /// # Arguments
    ///
    /// - `message_style` determines how the dialog will be drawn.
    /// - `message` is the message to display.
    ///
    pub fn new(message_style: MessageStyle, message: impl ToString) -> Self {
        // get the message size
        let message = message.to_string();
        debug_assert!(message.len() > 0, "new message is empty");
        let message_width = cmp::min(message.len(), 70);
        let rows = message_parse::ToRows::new(message_width).parse(&message);
        let message_size =
            Size { width: rows.iter().map(|row| row.len()).max().unwrap_or(0) as u16, height: rows.len() as u16 };
        // factor in the button size
        let button = ok_button().with_active();
        let button_size = button.size();
        // the dialog size need to reflect the border and margins
        let size = Size {
            width: 4 + cmp::max(cmp::max(message_size.width, button_size.width), " Information ".len() as u16),
            height: 4 + message_size.height + button_size.height,
        };

        Self { message: rows, message_style, button, size, catalog_type: CatalogType::MessageDialog }
    }
    /// Get the size of the message dialog.
    ///
    pub fn size(&self) -> Size {
        self.size
    }
    /// Consume a key pressed event. [ControlFlow::Continue] will be returned if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    pub fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("MessageDialog");
        match (key_event.modifiers, key_event.code) {
            (KeyModifiers::NONE, KeyCode::Esc | KeyCode::Enter) => break_event!(DialogResult::Exit)?,
            _ => match self.button.key_pressed(&key_event) {
                ControlFlow::Break(ControlResult::Selected(_)) => {
                    log::debug!("Ok pressed...");
                    break_event!(DialogResult::Exit)?
                }
                _ => {
                    beep();
                    break_event!(DialogResult::Continue)?;
                }
            },
        }
        ControlFlow::Continue(())
    }
    /// Draw the message dialog centered on the terminal screen. The cursor screen position will be returned.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the dialog can be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    pub fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("MessageDialog");
        let title = match self.message_style {
            MessageStyle::Error => " Error ",
            MessageStyle::Warning => " Warning ",
            MessageStyle::Normal => " Information ",
        };
        // add the dialog boiler plate
        let dialog_area = center(area, self.size);
        let dialog_styles = match self.message_style {
            MessageStyle::Error => self.catalog_type.get_styles(ControlState::Error),
            MessageStyle::Warning => self.catalog_type.get_styles(ControlState::Warning),
            MessageStyle::Normal => self.catalog_type.get_styles(ControlState::Normal),
        };
        Clear::default().render(dialog_area, buffer);
        Block::default()
            .borders(Borders::ALL)
            .title(Line::styled(title, dialog_styles.get(StyleId::DialogTitle)))
            .title_alignment(Alignment::Center)
            .border_style(dialog_styles.get(StyleId::DialogBorder))
            .render(dialog_area, buffer);
        // get area for message and button
        let client_area = inner_rect(dialog_area, (2, 2), (-2, -1));
        // show the message
        let message_height = self.message.len() as i32;
        let message_area = inner_rect(client_area, (0, 0), (0, message_height));
        let lines: Vec<Line> = self.message.iter().map(|s| Line::raw(s)).collect();
        let message_styles = self.catalog_type.get_styles(ControlState::Normal);
        Paragraph::new(lines)
            .style(message_styles.get(StyleId::Text))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .render(message_area, buffer);
        // show the button
        let button_size = self.button.size();
        let button_area = inner_rect(client_area, (0, -(button_size.height as i32)), (0, 0));
        let button_area = center(button_area, button_size);
        self.button.render(button_area, buffer, dialog_styles)
    }
}

mod message_parse {
    //! Isolate the message box text parsing to here. The functionality here needs to be updated.
    //! todo: move to the textwrap crate???

    /// The configuration used to parse a message.
    #[derive(Debug)]
    pub struct ToRows {
        // Sets the maximum line width.
        width: usize,
        // Any lines wider than the width will be truncated, otherwise wrapped
        truncate: bool,
        // Remove and leading or trailing whitespace.
        trim: bool,
    }
    #[allow(unused)]
    impl ToRows {
        pub fn new(width: usize) -> Self {
            Self { width, truncate: false, trim: false }
        }
        /// Parse the message.
        ///
        /// # Argument
        ///
        /// - `text` is the message that will be parsed.
        ///
        pub fn parse(&self, text: impl ToString) -> Vec<String> {
            RowParser::new(self).parse(text)
        }
    }

    /// The different types of text blocks.
    #[derive(Debug, PartialEq)]
    enum Chunks<'c> {
        /// A whitespace string.
        ///
        Whitespace(&'c str),
        /// A string that is not whitespace.
        ///
        Text(&'c str),
    }
    impl<'c> Chunks<'c> {
        /// Get the length of the text.
        fn len(&self) -> usize {
            match self {
                Chunks::Whitespace(chars) => chars.len(),
                Chunks::Text(chars) => chars.len(),
            }
        }
        /// Get the string content.
        ///
        fn text(&self) -> &'c str {
            match self {
                Chunks::Whitespace(chars) => chars,
                Chunks::Text(chars) => chars,
            }
        }
        /// Query if the text is whitespace.
        ///
        fn is_whitespace(&self) -> bool {
            match self {
                Chunks::Whitespace(_) => true,
                Chunks::Text(_) => false,
            }
        }
    }

    #[derive(Debug)]
    struct RowParser<'p> {
        /// The parser configuration.
        config: &'p ToRows,
        /// The resulting rows.
        rows: Vec<String>,
    }
    impl<'p> RowParser<'p> {
        /// Create a new instance of the row parser.
        ///
        /// # Arguments
        ///
        /// - `config` is the parser configuration.
        ///
        fn new(config: &'p ToRows) -> Self {
            Self { config, rows: vec![] }
        }
        /// Convert the string into a collection of strings depending on the parser configuration.
        ///
        /// # Arguments
        ///
        /// - `text` is the text string that will be parsed.
        ///
        fn parse(mut self, text: impl ToString) -> Vec<String> {
            for mut line in text.to_string().lines() {
                if self.config.trim {
                    line = line.trim();
                }
                if line.len() <= self.config.width {
                    self.rows.push(line.to_string());
                } else if self.config.truncate {
                    self.rows.push(format!("{}...", line[0..self.config.width - 3].to_string()));
                } else {
                    self.wrap_line(line);
                }
            }
            self.rows
        }
        /// Parse the string depending on the configuration.
        ///
        /// # Arguments
        ///
        /// - `line` is the line that will be parsed.
        ///
        fn wrap_line(&mut self, line: &str) {
            macro_rules! init_row {
                () => {
                    String::with_capacity(self.config.width)
                };
            }
            macro_rules! push {
                ($row:expr) => {
                    self.rows.push($row.trim_end().to_string())
                };
            }
            let mut row = init_row!();
            for chunk in get_chunks(line) {
                let chunk_len = chunk.len();
                if chunk_len > self.config.width {
                    // flush the current row and truncate the chunk text
                    if row.len() > 0 {
                        push!(row);
                        row = init_row!();
                    }
                    push!(format!("{}...", chunk.text()[..self.config.width - 3].to_string()));
                } else if row.len() + chunk_len < self.config.width {
                    if row.len() > 0 {
                        row.push_str(chunk.text())
                    } else if !chunk.is_whitespace() {
                        row.push_str(chunk.text());
                    }
                } else {
                    push!(row);
                    row = init_row!();
                    // because this is a new row don't add the separating whitespace
                    if !chunk.is_whitespace() {
                        row.push_str(chunk.text());
                    }
                }
            }
            if row.len() > 0 {
                self.rows.push(row.trim_end().to_string());
            }
        }
    }

    /// This will only be called when the line needs to be broke into chucks.
    ///
    /// # Arguments
    ///
    /// - `line` is the text that will be parsed.
    ///
    fn get_chunks(line: &str) -> Vec<Chunks> {
        let mut chunks: Vec<Chunks> = vec![];
        let mut chars = line.chars();
        // the parsing state
        let mut on_whitespace = chars.next().unwrap().is_whitespace();
        let mut start = 0;
        let mut end = start + 1;
        macro_rules! next {
            () => {{
                start = end;
                end = start + 1;
            }};
        }
        for ch in chars {
            match ch.is_whitespace() {
                true => match on_whitespace {
                    true => end += 1,
                    false => {
                        chunks.push(Chunks::Text(&line[start..end]));
                        on_whitespace = true;
                        next!()
                    }
                },
                false => match on_whitespace {
                    true => {
                        chunks.push(Chunks::Whitespace(&line[start..end]));
                        on_whitespace = false;
                        next!()
                    }
                    false => end += 1,
                },
            }
        }
        match on_whitespace {
            true => chunks.push(Chunks::Whitespace(&line[start..end])),
            false => chunks.push(Chunks::Text(&line[start..end])),
        }
        chunks
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn chunks() {
            let testcase = get_chunks("simple test   ");
            assert_eq!(testcase.len(), 4);
            assert_eq!(testcase[0], Chunks::Text("simple"));
            assert_eq!(testcase[1], Chunks::Whitespace(" "));
            assert_eq!(testcase[2], Chunks::Text("test"));
            assert_eq!(testcase[3], Chunks::Whitespace("   "));
            let testcase = get_chunks("   This is a test");
            assert_eq!(testcase.len(), 8);
            assert_eq!(testcase[0], Chunks::Whitespace("   "));
            assert_eq!(testcase[1], Chunks::Text("This"));
            assert_eq!(testcase[2], Chunks::Whitespace(" "));
            assert_eq!(testcase[3], Chunks::Text("is"));
            assert_eq!(testcase[4], Chunks::Whitespace(" "));
            assert_eq!(testcase[5], Chunks::Text("a"));
            assert_eq!(testcase[6], Chunks::Whitespace(" "));
            assert_eq!(testcase[7], Chunks::Text("test"));
        }
        #[test]
        fn trim() {
            // let config = ToRows::new(10).with_(true);
            let mut config = ToRows::new(10);
            config.trim = true;
            let testcase = RowParser::new(&config).parse("   trimmed  \n\t string  ");
            assert_eq!(testcase.len(), 2);
            assert_eq!(testcase[0], "trimmed");
            assert_eq!(testcase[1], "string");
        }
        #[test]
        fn truncate() {
            // let config = ToRows::new(10).with_truncate(true);
            let mut config = ToRows::new(10);
            config.truncate = true;
            let testcase = RowParser::new(&config).parse("This is 10\nThis is more");
            assert_eq!(testcase.len(), 2);
            assert_eq!(testcase[0], "This is 10");
            assert_eq!(testcase[1], "This is...");
        }
        #[test]
        fn wrap() {
            let config = ToRows::new(10);
            let testcase = RowParser::new(&config).parse("The tablespoons are dirty");
            assert_eq!(testcase.len(), 3);
            assert_eq!(testcase[0], "The");
            assert_eq!(testcase[1], "tablesp...");
            assert_eq!(testcase[2], "are dirty");
        }
    }
}
