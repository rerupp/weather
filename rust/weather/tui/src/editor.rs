//! A text field editor.
//!
//! The [Editor] is a modifiable text box. It can be configured to automatically lowercase or uppercase
//! characters and restrict input to specific characters.
//!

use super::label::{Label, LabelStyles, NO_SELECTOR};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Alignment,
    prelude::{Buffer, Position, Rect, Style},
    text::Line,
    widgets::{Clear, Widget},
};

/// The result of a control event.
#[derive(Debug, PartialOrd, PartialEq)]
pub enum EditorResult {
    /// Indicate the event was not consumed.
    Ignored,
    /// Indicate the event has consumed.
    Consumed,
    /// Indicate the event has consumed however it was not allowed.
    NotAllowed,
}

/// The styles used by the editor when rendering the editor.
///
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct EditorStyles {
    /// The style used to render the text being edited.
    pub text: Style,
    /// The styles to use when rendering the label.
    pub label: LabelStyles,
}
impl EditorStyles {
    /// The [Editor] default active styles.
    ///
    pub fn active() -> Self {
        Self { text: Style::default().underlined(), label: LabelStyles::active() }
    }

    /// The [Editor] default inactive styles.
    ///
    pub fn inactive() -> Self {
        Self { text: Style::default(), label: LabelStyles::inactive() }
    }

    /// The [Editor] default dim styles.
    ///
    pub fn dim() -> Self {
        Self { text: Style::default().dim().underlined(), label: LabelStyles::dim() }
    }
}

/// A basic configurable text line editor.
///
#[derive(Debug, Default)]
pub struct Editor {
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
    /// The editor styling.
    styles: EditorStyles,
    /// An optional identifier associated with the editor.
    id: Option<String>,
    /// The optional editor label.
    label: Option<Label>,
    /// The editor needs to know the width of a label there is no label.
    label_width: u16,
    /// Prevents changes to the text when true.
    readonly: bool,
    /// Used by the [EditorGroup](super::editor_group::EditorGroup) to track which editor in the collection
    /// is currently accepting key events.
    active: bool,
}
impl Editor {
    /// Create the editor seeding its contents with the text.
    ///
    /// # Arguments
    ///
    /// * `text` is the initial contents of the text editor.
    ///
    pub fn new(text: impl ToString) -> Self {
        let text = text.to_string();
        let position = text.len() as u16;
        Self { text, position, ..Self::default() }
    }

    /// A builder method that configures the text editor to always uppercase characters.
    ///
    pub fn with_uppercase(mut self) -> Self {
        self.uppercase = true;
        self.lowercase = !self.uppercase;
        self.text = self.text.to_uppercase();
        self
    }

    /// A builder method that configures the text editor to always lowercase characters.
    ///
    pub fn with_lowercase(mut self) -> Self {
        self.uppercase = false;
        self.lowercase = !self.uppercase;
        self.text = self.text.to_lowercase();
        self
    }

    /// A builder function that sets the maximum character width.
    ///
    /// # Arguments
    ///
    /// - `width` is the maximum number of characters the text can contain.
    ///
    pub fn with_width(mut self, width: u16) -> Self {
        if width < self.text.chars().count() as u16 {
            self.text = self.text.chars().take(width as usize).collect();
            self.position = width;
        }
        self.width = Some(width);
        self
    }

    /// Get the editors overall screen width.
    ///
    pub fn width(&self) -> u16 {
        // self.label_width()
        //     + match self.width {
        //         Some(width) => width,
        //         None => self.text.chars().count() as u16,
        //     }
        let label_width = self.label_width();
        let text_width = match self.width {
            Some(width) => width,
            None => self.text.chars().count() as u16,
        };
        label_width + text_width
    }

    /// Limit what characters are allowed to be added.
    ///
    /// # Arguments
    ///
    /// - `chars` are the characters that can be added.
    ///
    pub fn with_valid_chars(mut self, chars: impl Iterator<Item = char>) -> Self {
        self.valid_chars.replace(chars.collect());
        self
    }

    /// Make the editor readonly.
    ///
    pub fn with_readonly(mut self, readonly: bool) -> Self {
        // self.readonly = true;
        // self.active = false;
        self.readonly = readonly;
        if self.readonly {
            self.active = false;
        }
        self
    }

    /// Check if the editor is readonly.
    ///
    pub fn is_readonly(&self) -> bool {
        self.readonly
    }

    /// A builder method to set the editor state to active.
    ///
    pub fn with_active(mut self, active: bool) -> Self {
        self.set_active(active);
        self
    }

    /// Set whether the editor is active or not. The new state is ignored when the editor is readonly.
    ///
    /// # Arguments
    ///
    /// * `active` determines if the editor is active or not.
    ///
    pub fn set_active(&mut self, active: bool) {
        if self.is_readonly() {
            self.active = false;
        } else {
            self.active = active;
        }
    }

    /// Get the editor active state.
    ///
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// A builder method that adds an identifier name to the editor.
    ///
    /// # Arguments
    ///
    /// * `id` is the identifier name that will be associated with the editor.
    ///
    pub fn with_id(mut self, id: impl ToString) -> Self {
        self.id.replace(id.to_string());
        self
    }

    /// Get the editor identifier name.
    ///
    pub fn id(&self) -> &str {
        self.id.as_ref().map_or("", String::as_str)
    }

    /// A builder method that sets the style of the editor text and optional label.
    ///
    /// # Arguments
    ///
    /// * `styles` provides the style of editor text and label.
    ///
    pub fn with_styles(mut self, styles: EditorStyles) -> Self {
        self.set_styles(styles);
        self
    }

    /// Set the style of the editor text and optional label.
    ///
    /// # Arguments
    ///
    /// * `styles` provides the style of editor text and label.
    ///
    pub fn set_styles(&mut self, styles: EditorStyles) {
        self.styles = styles;
        if let Some(label) = self.label.as_mut() {
            label.set_styles(self.styles.label);
        }
    }

    /// Unit tests needs access to the styles
    ///
    #[cfg(test)]
    pub fn styles(&self) -> EditorStyles {
        self.styles
    }

    /// Add a left hand side text prompt to the editor.
    ///
    /// # Arguments
    ///
    /// * `label` is the left hand side prompt label.
    ///
    pub fn with_label(mut self, mut label: Label) -> Self {
        // the label always gets its style from the editor
        label.set_styles(self.styles.label);
        self.label.replace(label);
        self
    }

    /// A builder method that forces the width of the label.
    ///
    /// # Arguments
    ///
    /// * `width` is the new width of the label.
    ///
    pub fn with_label_width(mut self, width: u16) -> Self {
        self.set_label_width(width);
        self
    }

    /// Force the width of the label.
    ///
    /// # Arguments
    ///
    /// * `width` is the new width of the label.
    ///
    pub fn set_label_width(&mut self, width: u16) {
        self.label_width = width;
    }

    /// Get the editor labels screen width.
    ///
    pub fn label_width(&self) -> u16 {
        match self.label_width > 0 {
            true => self.label_width,
            false => self.label.as_ref().map_or(0, |label| label.width()),
        }
    }

    /// Unit tests need to access to the label.
    ///
    #[cfg(test)]
    pub fn label(&self) -> Option<&Label> {
        self.label.as_ref()
    }

    /// Set the alignment that will be used when rendering the label.
    ///
    /// # Arguments
    ///
    /// * `alignment` is how the labels text will be justified when rendered.
    ///
    pub fn set_label_alignment(&mut self, alignment: Alignment) {
        if let Some(label) = self.label.as_mut() {
            label.set_alignment(alignment);
        }
    }

    /// Get the label selector.
    ///
    pub fn label_selector(&self) -> char {
        self.label.as_ref().map_or(NO_SELECTOR, |label| label.selector())
    }

    /// Return the editor text contents.
    ///
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Dispatch a key pressed event to the text editor. The editor will return [EditorResult::Continue]
    /// if it does not consume the key event.
    ///
    /// # Arguments
    ///
    /// * `key_event` is a key pressed event.
    ///
    pub fn key_pressed(&mut self, key_event: &KeyEvent) -> EditorResult {
        match (key_event.modifiers, key_event.code) {
            (KeyModifiers::NONE, KeyCode::Home) => self.move_to_front(),
            (KeyModifiers::NONE, KeyCode::End) => self.move_to_end(),
            (KeyModifiers::NONE, KeyCode::Left) => self.move_left(),
            (KeyModifiers::CONTROL, KeyCode::Left) => self.move_to_front(),
            (KeyModifiers::NONE, KeyCode::Right) => self.move_right(),
            (KeyModifiers::CONTROL, KeyCode::Right) => self.move_to_end(),
            (KeyModifiers::NONE, KeyCode::Backspace) => self.delete_left(),
            (KeyModifiers::CONTROL, KeyCode::Backspace) => self.delete_all_left(),
            (KeyModifiers::NONE, KeyCode::Delete) => self.delete_right(),
            (KeyModifiers::CONTROL, KeyCode::Delete) => self.delete_all_right(),
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(ch)) => self.add(ch),
            _ => EditorResult::Ignored,
        }
    }

    /// Draw the text editor on the screen and return the current screen position.
    ///
    /// # Arguments
    ///
    /// * `area` is where on the screen the text editor will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    pub fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        Clear::default().render(area, buffer);

        // the label width set into the editor take precedence over the label size
        let label_width = match self.label_width > 0 {
            true => self.label_width,
            false => self.label.as_ref().map_or(0, |label| label.width()),
        };

        // if there is a label width always render the area
        if label_width > 0 {
            let label_area = Rect { x: area.x, y: area.y, width: label_width, height: 1 };
            match &self.label {
                None => Line::from("").render(label_area, buffer),
                Some(label) => label.render(label_area, buffer),
            }
        }

        // the editor area needs to take into account the labels width
        let mut editor_area =
            Rect { x: area.x + label_width, y: area.y, width: area.width.saturating_sub(label_width), height: 1 };
        if let Some(width) = self.width {
            editor_area.width = std::cmp::min(width, editor_area.width);
        }
        // Line::styled(&self.text, self.styles.text).render(editor_area, buffer);
        let text = match self.text.is_empty() {
            true => " ",
            false => self.text.as_str(),
        };
        Line::styled(text, self.styles.text).render(editor_area, buffer);
        match self.active {
            true => Some(Position::new(std::cmp::min(editor_area.x + self.position, area.width), area.y)),
            false => None,
        }
    }

    /// Move the current position to the first character, [EditorResult::NotAllowed] is returned
    /// if the position is already at the first character.
    ///
    fn move_to_front(&mut self) -> EditorResult {
        match self.position == 0 {
            true => EditorResult::NotAllowed,
            false => {
                self.position = 0;
                EditorResult::Consumed
            }
        }
    }

    /// Move the current position to the last character. [EditorResult::NotAllowed] is returned
    /// if the position is already at the last character.
    ///
    fn move_to_end(&mut self) -> EditorResult {
        let text_len = self.text.chars().count() as u16;
        let max_width = self.width.unwrap_or_else(|| text_len);
        match self.position >= max_width {
            true => EditorResult::NotAllowed,
            false => {
                self.position = text_len;
                EditorResult::Consumed
            }
        }
    }

    /// Delete all character left of the current position. [ControlResult::NotAllowed] will be returned
    /// if the position is at the first character.
    ///
    fn delete_all_left(&mut self) -> EditorResult {
        match self.position == 0 {
            true => EditorResult::NotAllowed,
            false => {
                self.text = self.text.chars().skip(self.position as usize).collect();
                self.position = 0;
                EditorResult::Consumed
            }
        }
    }

    /// Delete all character right of the current position. [EditorResult::NotAllowed] is returned
    /// if the position is at the last character.
    ///
    fn delete_all_right(&mut self) -> EditorResult {
        let text_len = self.text.chars().count() as u16;
        match self.position >= text_len {
            true => EditorResult::NotAllowed,
            false => {
                self.text = self.text.chars().take(self.position as usize).collect();
                EditorResult::Consumed
            }
        }
    }

    /// Move the current position 1 character to the left. [EditorResult::NotAllowed] is returned
    /// if the position is already at the first character.
    ///
    fn move_left(&mut self) -> EditorResult {
        match self.position == 0 {
            true => EditorResult::NotAllowed,
            false => {
                self.position -= 1;
                EditorResult::Consumed
            }
        }
    }

    /// Move the current position 1 character to the right. [ControlResult::NotAllowed] will be returned
    /// if the position is at the last character.
    ///
    fn move_right(&mut self) -> EditorResult {
        let text_width = self.text.chars().count() as u16;
        let max_width = match self.width {
            None => text_width,
            Some(width) => std::cmp::min(text_width, width),
        };
        match self.position >= max_width {
            true => EditorResult::NotAllowed,
            false => {
                self.position += 1;
                EditorResult::Consumed
            }
        }
    }

    /// Delete the character left of the current position. [EditorResult::NotAllowed] is returned
    /// if the current position is at the start of the string.
    ///
    fn delete_left(&mut self) -> EditorResult {
        match self.position == 0 {
            true => EditorResult::NotAllowed,
            false => {
                let mut iter = self.text.chars();
                let mut text = iter.by_ref().take(self.position as usize - 1).collect::<String>();
                iter.next();
                text.push_str(iter.as_str());
                self.text = text;
                self.position -= 1;
                EditorResult::Consumed
            }
        }
    }

    /// Delete the character right of the current position. [EditorResult::NotAllowed] is returned
    /// if the current position is at the end of the text.
    ///
    fn delete_right(&mut self) -> EditorResult {
        match self.position >= self.text.chars().count() as u16 {
            true => EditorResult::NotAllowed,
            false => {
                let mut iter = self.text.chars();
                let mut text = iter.by_ref().take(self.position as usize).collect::<String>();
                iter.next();
                text.push_str(iter.as_str());
                self.text = text;
                EditorResult::Consumed
            }
        }
    }

    /// A helper that adds a character to the text string. [EditorResult::NotAllowed] is returned
    /// if the character is not valid or the text width would be exceeded.
    ///
    fn add(&mut self, mut ch: char) -> EditorResult {
        // don't add the character if it would exceed the text width
        let text_len = self.text.chars().count() as u16;
        if let Some(width) = self.width {
            if text_len >= width {
                return EditorResult::NotAllowed;
            }
        }

        // adjust the character depending on the settings
        if self.lowercase {
            ch = ch.to_lowercase().next().unwrap();
        } else if self.uppercase {
            ch = ch.to_uppercase().next().unwrap();
        }

        // verify the character is allowed
        if !self.valid_chars.as_ref().map_or(true, |valid_chars| valid_chars.contains(ch)) {
            return EditorResult::NotAllowed;
        }

        // add the character
        match text_len == self.position {
            true => self.text.push(ch),
            false => {
                // insert the new character
                let mut iter = self.text.chars();
                let mut text = iter.by_ref().take(self.position as usize).collect::<String>();
                text.push(ch);
                text.push_str(iter.collect::<String>().as_str());
                self.text = text;
            }
        }

        // adjust the cursor position
        self.position += 1;

        EditorResult::Consumed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::Color;

    #[test]
    fn init() {
        let testcase = Editor::default();
        assert!(testcase.text.is_empty());
        assert_eq!(testcase.position, 0);
        assert_eq!(testcase.width, None);
        assert_eq!(testcase.uppercase, false);
        assert_eq!(testcase.lowercase, false);
        assert!(testcase.valid_chars.is_none());
        let testcase = Editor::new("testcase");
        assert_eq!(testcase.text, "testcase");
        assert_eq!(testcase.position, 8);
        assert_eq!(testcase.width, None);
        assert_eq!(testcase.uppercase, false);
        assert_eq!(testcase.lowercase, false);
        assert!(testcase.valid_chars.is_none());
        assert_eq!(Editor::default().with_width(10).width, Some(10));
        assert_eq!(Editor::default().with_uppercase().uppercase, true);
        assert_eq!(Editor::default().with_lowercase().lowercase, true);
        assert_eq!(Editor::default().with_valid_chars('a'..='c').valid_chars, Some("abc".to_string()));
    }

    #[test]
    fn readonly() {
        assert!(!Editor::default().is_readonly());
        assert!(Editor::default().with_readonly(true).is_readonly());
        assert!(!Editor::default().with_active(true).with_readonly(true).is_active());
    }

    #[test]
    fn active() {
        assert!(!Editor::default().is_active());
        assert!(Editor::default().with_active(true).is_active());
        assert!(!Editor::default().with_active(false).is_active());
        assert!(!Editor::default().with_readonly(true).with_active(true).is_active());
    }

    #[test]
    fn id() {
        assert!(Editor::default().id.is_none());
        assert_eq!(Editor::default().id(), "");
        assert_eq!(Editor::default().with_id("test").id(), "test");
    }

    #[test]
    fn styles() {
        let styles = EditorStyles {
            text: Style::default().red(),
            label: LabelStyles { text: Style::default().green(), selector: Style::default().blue() },
        };
        let mut testcase = Editor::default();
        assert_eq!(testcase.styles, EditorStyles::default());
        testcase.set_styles(styles);
        assert_eq!(testcase.styles, styles);
        assert_eq!(Editor::default().with_styles(styles).styles, styles);
    }

    #[test]
    fn label_styles() {
        assert_eq!(Editor::default().with_label(Label::default()).label.unwrap().styles(), LabelStyles::default());
        let styles = EditorStyles {
            text: Style::default().green(),
            label: LabelStyles { text: Style::default().blue(), selector: Style::default().red() },
        };

        // builder with styles before label
        assert_eq!(
            Editor::default().with_styles(styles).with_label(Label::default()).label.unwrap().styles(),
            styles.label
        );

        // builder with styles after label
        assert_eq!(
            Editor::default().with_label(Label::default()).with_styles(styles).label.unwrap().styles(),
            styles.label
        );

        // setting the styles
        let mut testcase = Editor::default().with_label(Label::default());
        testcase.set_styles(styles);
        assert_eq!(testcase.label.unwrap().styles(), styles.label);
    }

    #[test]
    fn label_width() {
        assert_eq!(Editor::default().label_width(), 0);
        assert_eq!(Editor::default().with_label_width(5).label_width(), 5);
        assert_eq!(Editor::default().with_label(Label::new("test")).label_width(), 4);
        assert_eq!(Editor::default().with_label_width(2).with_label(Label::new("test")).label_width(), 2);
    }

    #[test]
    fn text_casing() {
        let testcase = Editor::new("testcase").with_uppercase();
        assert_eq!(testcase.text, "TESTCASE");

        let testcase = Editor::new("TESTCASE").with_lowercase();
        assert_eq!(testcase.text, "testcase");
    }

    #[test]
    fn move_to_front() {
        let mut testcase = Editor::new("testcase");
        // move to the front
        assert_eq!(testcase.move_to_front(), EditorResult::Consumed);
        assert_eq!(testcase.position, 0);
        // since you're already there, this should fail
        assert_eq!(testcase.move_to_front(), EditorResult::NotAllowed);
        assert_eq!(testcase.position, 0);
    }

    #[test]
    fn move_to_end() {
        // check results without a width
        let mut testcase = Editor::new("testcase");
        assert_eq!(testcase.move_to_end(), EditorResult::NotAllowed);
        assert_eq!(testcase.position, 8);
        testcase.position = 7;
        assert_eq!(testcase.move_to_end(), EditorResult::Consumed);
        assert_eq!(testcase.position, 8);

        // check results with a width
        testcase.width.replace(9);
        assert_eq!(testcase.move_to_end(), EditorResult::Consumed);
        testcase.width.replace(7);
        assert_eq!(testcase.move_to_end(), EditorResult::NotAllowed);
    }

    #[test]
    fn delete_all_left() {
        let mut testcase = Editor::new("testcase");
        testcase.position = 4;
        assert_eq!(testcase.delete_all_left(), EditorResult::Consumed);
        assert_eq!(testcase.text, "case");
        assert_eq!(testcase.position, 0);
        assert_eq!(testcase.delete_all_left(), EditorResult::NotAllowed);
        assert_eq!(testcase.position, 0);
        assert_eq!(testcase.text, "case");
    }

    #[test]
    fn delete_all_right() {
        // the first test case is without a text field width
        let mut testcase = Editor::new("testcase");
        testcase.position = 4;
        assert_eq!(testcase.delete_all_right(), EditorResult::Consumed);
        assert_eq!(testcase.text, "test");
        assert_eq!(testcase.position, 4);
        assert_eq!(testcase.delete_all_right(), EditorResult::NotAllowed);
        assert_eq!(testcase.text, "test");
        assert_eq!(testcase.position, 4);

        // this testcase uses a text field width
        let mut testcase = Editor::new("testcase").with_width(8);
        testcase.position = 4;
        assert_eq!(testcase.delete_all_right(), EditorResult::Consumed);
        assert_eq!(testcase.text, "test");
        assert_eq!(testcase.position, 4);
        assert_eq!(testcase.delete_all_right(), EditorResult::NotAllowed);
        assert_eq!(testcase.text, "test");
        assert_eq!(testcase.position, 4);
    }

    #[test]
    fn move_left() {
        let mut testcase = Editor::new("testcase");
        testcase.position = 2;
        assert_eq!(testcase.move_left(), EditorResult::Consumed);
        assert_eq!(testcase.position, 1);
        assert_eq!(testcase.move_left(), EditorResult::Consumed);
        assert_eq!(testcase.position, 0);
        assert_eq!(testcase.move_left(), EditorResult::NotAllowed);
        assert_eq!(testcase.position, 0);
    }

    #[test]
    fn move_right() {
        // no width
        let mut testcase = Editor::new("testcase");
        testcase.position = 6;
        assert_eq!(testcase.move_right(), EditorResult::Consumed);
        assert_eq!(testcase.position, 7);
        assert_eq!(testcase.move_right(), EditorResult::Consumed);
        assert_eq!(testcase.position, 8);
        assert_eq!(testcase.move_right(), EditorResult::NotAllowed);
        assert_eq!(testcase.position, 8);

        // with width
        testcase.width.replace(7);
        testcase.position = 5;
        assert_eq!(testcase.move_right(), EditorResult::Consumed);
        assert_eq!(testcase.position, 6);
        assert_eq!(testcase.move_right(), EditorResult::Consumed);
        assert_eq!(testcase.position, 7);
        assert_eq!(testcase.move_right(), EditorResult::NotAllowed);
        assert_eq!(testcase.position, 7);

        // make sure the editor doesn't move past the last character
        let mut testcase = Editor::new("a").with_width(5);
        assert_eq!(testcase.move_right(), EditorResult::NotAllowed);
        testcase.position = 0;
        assert_eq!(testcase.move_right(), EditorResult::Consumed);
    }

    #[test]
    fn delete_left() {
        let mut testcase = Editor::new("testcase");
        testcase.position = 2;
        assert_eq!(testcase.delete_left(), EditorResult::Consumed);
        assert_eq!(testcase.position, 1);
        assert_eq!(testcase.text, "tstcase");
        assert_eq!(testcase.delete_left(), EditorResult::Consumed);
        assert_eq!(testcase.position, 0);
        assert_eq!(testcase.text, "stcase");
        assert_eq!(testcase.delete_left(), EditorResult::NotAllowed);
        assert_eq!(testcase.position, 0);
        assert_eq!(testcase.text, "stcase");
    }

    #[test]
    fn delete_right() {
        // no width
        let mut testcase = Editor::new("testcase");
        testcase.position = 6;
        assert_eq!(testcase.delete_right(), EditorResult::Consumed);
        assert_eq!(testcase.position, 6);
        assert_eq!(testcase.text, "testcae");
        assert_eq!(testcase.delete_right(), EditorResult::Consumed);
        assert_eq!(testcase.position, 6);
        assert_eq!(testcase.text, "testca");
        assert_eq!(testcase.delete_right(), EditorResult::NotAllowed);
        assert_eq!(testcase.position, 6);
        assert_eq!(testcase.text, "testca");
    }

    #[test]
    fn add() {
        // simple add
        let mut testcase = Editor::default();
        assert_eq!(testcase.add('t'), EditorResult::Consumed);
        assert_eq!(testcase.text, "t");
        assert_eq!(testcase.position, 1);
        assert_eq!(testcase.add('e'), EditorResult::Consumed);
        assert_eq!(testcase.text, "te");
        assert_eq!(testcase.position, 2);
        assert_eq!(testcase.add('s'), EditorResult::Consumed);
        assert_eq!(testcase.text, "tes");
        assert_eq!(testcase.position, 3);
        assert_eq!(testcase.add('t'), EditorResult::Consumed);
        assert_eq!(testcase.text, "test");
        assert_eq!(testcase.position, 4);

        // insert
        testcase.position = 0;
        assert_eq!(testcase.add(' '), EditorResult::Consumed);
        assert_eq!(testcase.text, " test");
        assert_eq!(testcase.position, 1);

        // force uppercase
        let mut testcase = Editor::default().with_uppercase();
        assert_eq!(testcase.add('t'), EditorResult::Consumed);
        assert_eq!(testcase.text, "T");
        assert_eq!(testcase.position, 1);

        // force lowercase
        let mut testcase = Editor::default().with_lowercase();
        testcase.lowercase = true;
        assert_eq!(testcase.add('T'), EditorResult::Consumed);
        assert_eq!(testcase.text, "t");
        assert_eq!(testcase.position, 1);

        // validate chars
        let mut testcase = Editor::default().with_valid_chars("ab".chars().chain("B".chars()));
        assert_eq!(testcase.add('a'), EditorResult::Consumed);
        assert_eq!(testcase.text, "a");
        assert_eq!(testcase.position, 1);
        assert_eq!(testcase.add('b'), EditorResult::Consumed);
        assert_eq!(testcase.text, "ab");
        assert_eq!(testcase.position, 2);
        assert_eq!(testcase.add('A'), EditorResult::NotAllowed);
        assert_eq!(testcase.text, "ab");
        assert_eq!(testcase.position, 2);
        assert_eq!(testcase.add('B'), EditorResult::Consumed);
        assert_eq!(testcase.text, "abB");
        assert_eq!(testcase.position, 3);

        // width tests

        // add
        let mut testcase = Editor::default().with_width(3);
        assert_eq!(testcase.add('a'), EditorResult::Consumed);
        assert_eq!(testcase.text, "a");
        assert_eq!(testcase.position, 1);
        assert_eq!(testcase.add('b'), EditorResult::Consumed);
        assert_eq!(testcase.text, "ab");
        assert_eq!(testcase.position, 2);
        assert_eq!(testcase.add('c'), EditorResult::Consumed);
        assert_eq!(testcase.text, "abc");
        assert_eq!(testcase.position, 3);
        assert_eq!(testcase.text, "abc");
        assert_eq!(testcase.position, 3);
        assert_eq!(testcase.add('d'), EditorResult::NotAllowed);
        assert_eq!(testcase.text, "abc");
        assert_eq!(testcase.position, 3);

        // insert
        testcase.width.replace(4);
        testcase.position = 0;
        assert_eq!(testcase.add(' '), EditorResult::Consumed);
        assert_eq!(testcase.text, " abc");
        assert_eq!(testcase.position, 1);
        assert_eq!(testcase.add(' '), EditorResult::NotAllowed);
        assert_eq!(testcase.text, " abc");
        assert_eq!(testcase.position, 1);

        // force uppercase
        let mut testcase = Editor::default().with_uppercase().with_width(1);
        assert_eq!(testcase.add('u'), EditorResult::Consumed);
        assert_eq!(testcase.text, "U");
        assert_eq!(testcase.position, 1);
        assert_eq!(testcase.add(' '), EditorResult::NotAllowed);
        assert_eq!(testcase.text, "U");
        assert_eq!(testcase.position, 1);

        // force lowercase
        let mut testcase = Editor::default().with_lowercase().with_width(1);
        assert_eq!(testcase.add('L'), EditorResult::Consumed);
        assert_eq!(testcase.text, "l");
        assert_eq!(testcase.position, 1);
        assert_eq!(testcase.add(' '), EditorResult::NotAllowed);
        assert_eq!(testcase.text, "l");
        assert_eq!(testcase.position, 1);

        // validate chars
        // the editor restricts adding characters not the initial seed
        let mut testcase = Editor::new("A").with_valid_chars("B".chars()).with_width(2);
        assert_eq!(testcase.add('B'), EditorResult::Consumed);
        assert_eq!(testcase.text, "AB");
        assert_eq!(testcase.position, 2);
        assert_eq!(testcase.add('B'), EditorResult::NotAllowed);
        assert_eq!(testcase.text, "AB");
        assert_eq!(testcase.position, 2);
    }

    #[test]
    fn width() {
        assert_eq!(Editor::default().with_width(4).width(), 4);
        assert_eq!(Editor::new("testcase").width(), "testcase".len() as u16);
        assert_eq!(Editor::new("test").with_width(6).width(), 6);
    }

    #[test]
    fn render() {
        let area = Rect::new(0, 0, 5, 1);
        let mut buffer = Buffer::empty(area);
        macro_rules! assert_cell {
            ($x: literal, $value:expr, $style:expr) => {
                assert_eq!(buffer[($x, 0)].symbol(), $value);
                assert_eq!(buffer[($x, 0)].style(), $style);
            };
        }
        let default_cell = &buffer[(0, 0)].clone();
        let styles = EditorStyles {
            text: Style::default().yellow().bg(Color::Reset).underline_color(Color::Reset),
            label: LabelStyles::default(),
        };
        let editor_text = "abc";

        // no width
        let position = Editor::new(editor_text).with_styles(styles).render(area, &mut buffer);
        assert_eq!(position, None);
        assert_cell!(0, "a", styles.text);
        assert_cell!(1, "b", styles.text);
        assert_cell!(2, "c", styles.text);
        assert_cell!(3, " ", styles.text);
        assert_cell!(4, " ", styles.text);

        // with width
        buffer.reset();
        let position =
            Editor::new(editor_text).with_styles(styles).with_active(true).with_width(3).render(area, &mut buffer);
        assert_eq!(position.unwrap(), Position { x: 3, y: 0 });
        assert_cell!(0, "a", styles.text);
        assert_cell!(1, "b", styles.text);
        assert_cell!(2, "c", styles.text);
        assert_cell!(3, " ", default_cell.style());
        assert_cell!(4, " ", default_cell.style());

        // with label width
        buffer.reset();
        let position = Editor::new(editor_text)
            .with_styles(styles)
            .with_active(true)
            .with_width(3)
            .with_label_width(2)
            .render(area, &mut buffer);
        assert_eq!(position.unwrap(), Position { x: 5, y: 0 });
        assert_cell!(0, " ", default_cell.style());
        assert_cell!(1, " ", default_cell.style());
        assert_cell!(2, "a", styles.text);
        assert_cell!(3, "b", styles.text);
        assert_cell!(4, "c", styles.text);

        // with a label width forcing the editor to be truncated
        buffer.reset();
        let position =
            Editor::new(editor_text).with_styles(styles).with_width(3).with_label_width(3).render(area, &mut buffer);
        assert_eq!(position, None);
        assert_cell!(0, " ", default_cell.style());
        assert_cell!(1, " ", default_cell.style());
        assert_cell!(2, " ", default_cell.style());
        assert_cell!(3, "a", styles.text);
        assert_cell!(4, "b", styles.text);

        // include a label
        buffer.reset();
        let styles_with_label = EditorStyles {
            text: styles.text,
            label: LabelStyles {
                text: Style::new().cyan().bg(Color::Reset).underline_color(Color::Reset),
                selector: Style::default(),
            },
        };
        let mut editor = Editor::new(editor_text)
            .with_styles(styles_with_label)
            .with_width(3)
            .with_active(true)
            .with_label(Label::new("p:"));
        assert_eq!(editor.render(area, &mut buffer).unwrap(), Position { x: 5, y: 0 });
        assert_cell!(0, "p", styles_with_label.label.text);
        assert_cell!(1, ":", styles_with_label.label.text);
        assert_cell!(2, "a", styles.text);
        assert_cell!(3, "b", styles.text);
        assert_cell!(4, "c", styles.text);

        // make sure the styles change
        editor.set_styles(EditorStyles::default());
        editor.render(area, &mut buffer);
        let default_buffer_style = Style::new().fg(Color::Reset).bg(Color::Reset).underline_color(Color::Reset);
        assert_cell!(0, "p", default_buffer_style);
        assert_cell!(1, ":", default_buffer_style);
        assert_cell!(2, "a", default_buffer_style);
        assert_cell!(3, "b", default_buffer_style);
        assert_cell!(4, "c", default_buffer_style);
    }
}
