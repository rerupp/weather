//! The general text editor implementation.
//!
//! The [TextEditor] is a modifiable text box. The `TextEditor` can be configured to
//! automatically lowercase or uppercase characters. It can also be configured to
//! restrict input to specific characters.
//!
use super::*;

/// A basic, configurable text editor.
#[derive(Debug, Default)]
pub struct TextEditor {
    /// The text being edited.
    text: String,
    /// The current position within the text.
    position: u16,
    /// The maximum width of the text content.
    width: Option<u16>,
    /// Force text to be uppercase.
    uppercase: bool,
    /// Force text to be lowercase.
    lowercase: bool,
    /// Limit the allowed text to these characters.
    valid_chars: Option<String>,
}
impl TextEditor {
    /// A builder function that sets the maximum character width.
    ///
    /// # Arguments
    ///
    /// - `width` is the maximum number of characters the text can contain.
    ///
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }
    /// Configure the text editor to uppercase text characters.
    ///
    pub fn with_uppercase_only(mut self) -> Self {
        self.uppercase = true;
        self.lowercase = false;
        self
    }
    /// Configure the text editor to lowercase text characters.
    ///
    pub fn with_lowercase_only(mut self) -> Self {
        self.lowercase = true;
        self.uppercase = false;
        self
    }
    /// Limit the text characters accepted. This is an append operation not an overwrite.
    ///
    /// # Arguments
    ///
    /// - `chars` are the characters that can be added to the text.
    ///
    pub fn with_valid_chars(mut self, chars: impl Iterator<Item = char>) -> Self {
        if self.valid_chars.is_some() {
            let valid_chars = self.valid_chars.take().unwrap();
            self.valid_chars.replace(valid_chars.chars().chain(chars).collect());
        } else {
            self.valid_chars.replace(chars.collect());
        }
        self
    }
    /// Sets the initial text contents. There is no validation of the text so GIGO.
    ///
    /// # Arguments
    ///
    /// - `text` is the initial text field content.
    ///
    pub fn with_text(mut self, text: impl ToString) -> Self {
        self.text = text.to_string();
        self
    }
    /// Move the current position to the first character, [ControlResult::NotAllowed] will be returned
    /// if the position is already at the first character.
    fn move_to_front(&mut self) -> ControlFlow<ControlResult> {
        match self.position == 0 {
            true => break_event!(ControlResult::NotAllowed),
            false => {
                self.position = 0;
                break_event!(ControlResult::Continue)
            }
        }
    }
    /// Move the current position to the last character, [ControlResult::NotAllowed] will be returned
    /// if the position is already at the last character.
    fn move_to_end(&mut self) -> ControlFlow<ControlResult> {
        macro_rules! set_position {
            ($position:expr) => {{
                if self.position == $position {
                    break_event!(ControlResult::NotAllowed)
                } else {
                    self.position = $position;
                    break_event!(ControlResult::Continue)
                }
            }};
        }
        match self.width {
            None => set_position!(self.text.len() as u16),
            // the position always needs to less than the width
            Some(width) => set_position!(cmp::min(width - 1, self.text.len() as u16)),
        }
    }
    /// Delete all character left of the current position. [ControlResult::NotAllowed] will be returned
    /// if the position is at the first character.
    fn delete_all_left(&mut self) -> ControlFlow<ControlResult> {
        match self.position == 0 {
            true => break_event!(ControlResult::NotAllowed),
            false => {
                self.text = self.text.chars().skip(self.position as usize).collect();
                self.position = 0;
                break_event!(ControlResult::Continue)
            }
        }
    }
    /// Delete all character right of the current position. [ControlResult::NotAllowed] will be returned
    /// if the position is at the last character.
    fn delete_all_right(&mut self) -> ControlFlow<ControlResult> {
        macro_rules! delete_all_right {
            ($position:expr) => {
                if self.position == $position {
                    // ControlFlow::Break(ControlResult::KeyMunched(false))
                    break_event!(ControlResult::NotAllowed)
                } else {
                    self.text = self.text.chars().take(self.position as usize).collect();
                    // ControlFlow::Break(ControlResult::KeyMunched(true))
                    break_event!(ControlResult::Continue)
                }
            };
        }
        match self.width {
            None => delete_all_right!(self.text.len() as u16),
            Some(width) => delete_all_right!(cmp::min(width, self.text.len() as u16)),
        }
    }
    /// Move the current position 1 character to the left. [ControlResult::NotAllowed] will be returned
    /// if the position is at the first character.
    fn move_left(&mut self) -> ControlFlow<ControlResult> {
        if self.position == 0 {
            break_event!(ControlResult::NotAllowed)
        } else {
            self.position -= 1;
            break_event!(ControlResult::Continue)
        }
    }
    /// Move the current position 1 character to the right. [ControlResult::NotAllowed] will be returned
    /// if the position is at the last character.
    fn move_right(&mut self) -> ControlFlow<ControlResult> {
        macro_rules! move_right {
            ($width:expr) => {
                if self.position == $width {
                    break_event!(ControlResult::NotAllowed)
                } else {
                    self.position = cmp::min($width, self.position + 1);
                    break_event!(ControlResult::Continue)
                }
            };
        }
        match self.width {
            None => move_right!(self.text.len() as u16),
            Some(width) => move_right!(width.saturating_sub(1)),
        }
    }
    /// Delete the character left of the current position. [ControlResult::NotAllowed] will be returned
    /// if the current position is at the first character.
    fn delete_left(&mut self) -> ControlFlow<ControlResult> {
        if self.position == 0 {
            break_event!(ControlResult::NotAllowed)
        } else {
            let head = self.text.chars().take(self.position.saturating_sub(1) as usize);
            let tail = self.text.chars().skip(self.position as usize);
            self.text = head.chain(tail).collect();
            self.position -= 1;
            break_event!(ControlResult::Continue)
        }
    }
    /// Delete the character right of the current position. [ControlResult::NotAllowed] will be returned
    /// if the current position is at the end of the text.
    fn delete_right(&mut self) -> ControlFlow<ControlResult> {
        macro_rules! delete_right {
            ($not_ok:expr) => {
                if $not_ok {
                    // ControlFlow::Break(ControlResult::KeyMunched(false))
                    break_event!(ControlResult::NotAllowed)
                } else {
                    let head = self.text.chars().take(self.position as usize);
                    let tail = self.text.chars().skip(self.position as usize + 1);
                    self.text = head.chain(tail).collect();
                    // ControlFlow::Break(ControlResult::KeyMunched(true))
                    break_event!(ControlResult::Continue)
                }
            };
        }
        match self.width {
            None => delete_right!(self.position == self.text.len() as u16),
            Some(width) => delete_right!(self.position == (width - 1) && (self.text.len() as u16) < width),
        }
    }
    /// A helper that will validate the character. [ControlResult::NotAllowed] will be returned
    /// if the character is not valid.
    fn prepare_char(&self, mut ch: char) -> ControlFlow<ControlResult, char> {
        if self.uppercase {
            ch = ch.to_uppercase().next().unwrap();
        } else if self.lowercase {
            ch = ch.to_lowercase().next().unwrap();
        }
        match self.valid_chars.as_ref().map_or(true, |valid_chars| valid_chars.contains(ch)) {
            true => ControlFlow::Continue(ch),
            false => break_event!(ControlResult::NotAllowed),
        }
    }
    /// A helper that adds a character to the text. [ControlResult::NotAllowed] will be returned
    /// if the character is not valid or the maximum text length would be exceeded.
    fn add(&mut self, mut ch: char) -> ControlFlow<ControlResult> {
        macro_rules! insert_char {
            ($ch:expr) => {{
                let mut text: String = self.text.chars().take(self.position as usize).collect();
                text.push($ch);
                self.text.chars().skip(self.position as usize).for_each(|ch| text.push(ch));
                self.text = text;
                self.position += 1;
                break_event!(ControlResult::Continue)
            }};
        }
        ch = self.prepare_char(ch)?;
        match self.width {
            None => insert_char!(ch),
            Some(width) => {
                // special case being at the end of the field
                if self.position == width - 1 {
                    self.text = self.text.chars().take(self.position as usize).collect();
                    self.text.push(ch);
                    break_event!(ControlResult::Continue)
                } else if self.text.len() < width as usize {
                    insert_char!(ch)
                } else {
                    break_event!(ControlResult::NotAllowed)
                }
            }
        }
    }
}
impl FieldEditor for TextEditor {
    /// Get the text editor screen size.
    fn size(&self) -> Size {
        Size {
            width: match self.width {
                None => self.text.len() as u16,
                Some(width) => width,
            },
            height: 1,
        }
    }
    /// Dispatch a key pressed event to the text editor and return the result. The editor
    /// will return [ControlFlow::Continue] if it does not consume the event.
    ///
    /// # Arguments
    ///
    /// * `key_event` is a key pressed event.
    ///
    fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!("TextEditor");
        match (key_event.modifiers, key_event.code) {
            // all matches return ControlResult Continue or NotAllowed
            (KeyModifiers::CONTROL, KeyCode::Left) => self.move_to_front(),
            (KeyModifiers::CONTROL, KeyCode::Right) => self.move_to_end(),
            (KeyModifiers::CONTROL, KeyCode::Backspace) => self.delete_all_left(),
            (KeyModifiers::CONTROL, KeyCode::Delete) => self.delete_all_right(),
            (KeyModifiers::NONE, KeyCode::Home) => self.move_to_front(),
            (KeyModifiers::NONE, KeyCode::End) => self.move_to_end(),
            (KeyModifiers::NONE, KeyCode::Left) => self.move_left(),
            (KeyModifiers::NONE, KeyCode::Right) => self.move_right(),
            (KeyModifiers::NONE, KeyCode::Backspace) => self.delete_left(),
            (KeyModifiers::NONE, KeyCode::Delete) => self.delete_right(),
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(ch)) => self.add(ch),
            // pass the key event along
            _ => ControlFlow::Continue(()),
        }
    }
    /// Draw the text editor on the screen and return the current screen position.
    ///
    /// # Arguments
    ///
    /// * `area` is where on the screen the text editor will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// * `styles` is catalog that will be used to render the text editor.
    ///
    fn render(&self, mut area: Rect, buffer: &mut Buffer, styles: &StyleCatalog) -> Position {
        log_render!("TextEditor");
        let text = match self.width {
            None => self.text.clone(),
            // pad the edit text if there is a width
            Some(width) => format!("{:<field_width$}", self.text, field_width = width as usize),
        };
        area.width = cmp::min(area.width, text.len() as u16);
        Paragraph::new(text).style(styles.get(StyleId::Text)).render(area, buffer);
        Position::new(area.x + self.position, area.y)
    }
    /// Return the editor text contents.
    ///
    fn text(&self) -> &str {
        &self.text
    }
    /// Set the editor text contents.
    ///
    /// # Arguments
    ///
    /// * `content` is the text set into the editor.
    ///
    fn set_text(&mut self, text: impl ToString) {
        self.text = text.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_to_front() {
        let mut testcase = TextEditor::default();
        testcase.text = "testcase".to_string();
        testcase.position = testcase.text.len() as u16;
        // move to the front
        assert_eq!(testcase.move_to_front(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 0);
        // since you're already there, this should fail
        assert_eq!(testcase.move_to_front(), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.position, 0);
    }

    #[test]
    fn move_to_end() {
        let mut testcase = TextEditor::default();
        testcase.text = "testcase".to_string();
        testcase.position = 0;
        assert_eq!(testcase.move_to_end(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 8);
        // since you're already there this should not be allowed
        assert_eq!(testcase.move_to_end(), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.position, 8);
        // set the width to be wider than the text
        testcase.width.replace(12);
        testcase.position = 3;
        assert_eq!(testcase.move_to_end(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 8);
        // set the width to match the text width
        testcase.width.replace(8);
        testcase.position = 3;
        assert_eq!(testcase.move_to_end(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 7);
        assert_eq!(testcase.move_to_end(), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.position, 7);
    }

    #[test]
    fn delete_all_left() {
        let mut testcase = TextEditor::default();
        testcase.text = "testcase".to_string();
        testcase.position = 4;
        assert_eq!(testcase.delete_all_left(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.text, "case");
        assert_eq!(testcase.position, 0);
        assert_eq!(testcase.delete_all_left(), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.position, 0);
        assert_eq!(testcase.text, "case");
    }

    #[test]
    fn delete_all_right() {
        // the first test case is without a text field width
        let mut testcase = TextEditor::default();
        testcase.text = "testcase".to_string();
        testcase.position = 4;
        assert_eq!(testcase.delete_all_right(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.text, "test");
        assert_eq!(testcase.position, 4);
        assert_eq!(testcase.delete_all_right(), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.text, "test");
        assert_eq!(testcase.position, 4);
        // this testcase uses a text field width
        let mut testcase = TextEditor::default();
        testcase.text = "testcase".to_string();
        testcase.position = 4;
        testcase.width = Some(10);
        assert_eq!(testcase.delete_all_right(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.text, "test");
        assert_eq!(testcase.position, 4);
        assert_eq!(testcase.delete_all_right(), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.text, "test");
        assert_eq!(testcase.position, 4);
    }

    #[test]
    fn move_left() {
        let mut testcase = TextEditor::default();
        testcase.text = "testcase".to_string();
        testcase.position = 2;
        assert_eq!(testcase.move_left(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 1);
        assert_eq!(testcase.move_left(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 0);
        assert_eq!(testcase.move_left(), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.position, 0);
    }

    #[test]
    fn move_right() {
        // no width
        let mut testcase = TextEditor::default();
        testcase.text = "testcase".to_string();
        testcase.position = 6;
        assert_eq!(testcase.move_right(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 7);
        assert_eq!(testcase.move_right(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 8);
        assert_eq!(testcase.move_right(), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.position, 8);
        // has width
        testcase.width = Some(8);
        testcase.position = 5;
        assert_eq!(testcase.move_right(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 6);
        assert_eq!(testcase.move_right(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 7);
        assert_eq!(testcase.move_right(), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.position, 7);
    }

    #[test]
    fn delete_left() {
        let mut testcase = TextEditor::default();
        testcase.text = "testcase".to_string();
        testcase.position = 5;
        assert_eq!(testcase.delete_left(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 4);
        assert_eq!(testcase.delete_left(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 3);
        assert_eq!(testcase.delete_left(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 2);
        assert_eq!(testcase.text, "tease");
        testcase.position = 1;
        assert_eq!(testcase.delete_left(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 0);
        assert_eq!(testcase.text, "ease");
        assert_eq!(testcase.delete_left(), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.position, 0);
    }

    #[test]
    fn delete_right() {
        // no width
        let mut testcase = TextEditor::default();
        testcase.text = "testcase".to_string();
        testcase.position = 2;
        assert_eq!(testcase.delete_right(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 2);
        assert_eq!(testcase.delete_right(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 2);
        assert_eq!(testcase.delete_right(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 2);
        assert_eq!(testcase.text, "tease");
        testcase.position = testcase.text.len() as u16;
        assert_eq!(testcase.delete_right(), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.position, testcase.text.len() as u16);
        assert_eq!(testcase.text, "tease");
        // with width
        testcase.text = "testcases".to_string();
        testcase.width.replace(9);
        testcase.position = 8;
        assert_eq!(testcase.delete_right(), break_event!(ControlResult::Continue));
        assert_eq!(testcase.position, 8);
        assert_eq!(testcase.text, "testcase");
        assert_eq!(testcase.delete_right(), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.position, 8);
        assert_eq!(testcase.text, "testcase");
    }

    #[test]
    fn prepare_char() {
        // no conversions
        let mut testcase = TextEditor::default();
        assert_eq!(testcase.prepare_char('t'), ControlFlow::Continue('t'));
        testcase.valid_chars = Some("s".to_string());
        assert_eq!(testcase.prepare_char('t'), ControlFlow::Break(ControlResult::NotAllowed));
        testcase.valid_chars = Some("start".to_string());
        assert_eq!(testcase.prepare_char('t'), ControlFlow::Continue('t'));
        // uppercase
        let mut testcase = TextEditor::default();
        testcase.uppercase = true;
        assert_eq!(testcase.prepare_char('t'), ControlFlow::Continue('T'));
        // Is this a valid use case?
        testcase.valid_chars = Some("test".to_string());
        assert_eq!(testcase.prepare_char('t'), ControlFlow::Break(ControlResult::NotAllowed));
        testcase.valid_chars = Some("START".to_string());
        assert_eq!(testcase.prepare_char('t'), ControlFlow::Continue('T'));
        // lowercase
        let mut testcase = TextEditor::default();
        testcase.lowercase = true;
        assert_eq!(testcase.prepare_char('T'), ControlFlow::Continue('t'));
        testcase.valid_chars = Some("TEST".to_string());
        // Is this a valid use case?
        assert_eq!(testcase.prepare_char('T'), ControlFlow::Break(ControlResult::NotAllowed));
        testcase.valid_chars = Some("test".to_string());
        assert_eq!(testcase.prepare_char('t'), ControlFlow::Continue('t'));
    }

    #[test]
    fn add() {
        // no width
        // simple add
        let mut testcase = TextEditor::default();
        assert_eq!(testcase.add('t'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.add('e'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.add('s'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.add('t'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.text, "test");
        assert_eq!(testcase.position, 4);
        // insert
        let mut testcase = TextEditor::default();
        testcase.text = "tease".to_string();
        testcase.position = 2;
        assert_eq!(testcase.add('s'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.add('t'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.add('c'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.text, "testcase");
        assert_eq!(testcase.position, 5);
        // force uppercase
        let mut testcase = TextEditor::default();
        testcase.uppercase = true;
        assert_eq!(testcase.add('t'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.text, "T");
        // force lowercase
        let mut testcase = TextEditor::default();
        testcase.lowercase = true;
        assert_eq!(testcase.add('T'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.text, "t");
        // validate chars
        let mut testcase = TextEditor::default();
        testcase.valid_chars = Some("test".to_string());
        assert_eq!(testcase.add('T'), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.add('t'), break_event!(ControlResult::Continue));

        // width tests

        // add
        let mut testcase = TextEditor::default();
        testcase.width = Some(4);
        assert_eq!(testcase.add('t'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.add('e'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.add('s'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.add('t'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.text, "test");
        assert_eq!(testcase.position, 3);
        assert_eq!(testcase.add('s'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.text, "tess");
        assert_eq!(testcase.position, 3);
        // insert
        let mut testcase = TextEditor::default();
        testcase.width = Some(8);
        testcase.text = "tease".to_string();
        testcase.position = 2;
        assert_eq!(testcase.add('s'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.add('t'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.add('c'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.text, "testcase");
        assert_eq!(testcase.position, 5);
        assert_eq!(testcase.add('s'), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.text, "testcase");
        assert_eq!(testcase.position, 5);
        // force uppercase
        let mut testcase = TextEditor::default();
        testcase.width = Some(2);
        testcase.uppercase = true;
        assert_eq!(testcase.add('t'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.text, "T");
        // force lowercase
        let mut testcase = TextEditor::default();
        testcase.lowercase = true;
        testcase.width = Some(2);
        assert_eq!(testcase.add('T'), break_event!(ControlResult::Continue));
        assert_eq!(testcase.text, "t");
        // validate chars
        let mut testcase = TextEditor::default();
        testcase.width = Some(2);
        testcase.valid_chars = Some("test".to_string());
        assert_eq!(testcase.add('T'), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.add('t'), break_event!(ControlResult::Continue));
    }
}
