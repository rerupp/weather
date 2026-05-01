//! A vertical collection of editors.

use super::editor::{Editor, EditorResult, EditorStyles};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::prelude::{Alignment, Buffer, Position, Rect, Size};

/// The active and inactive styles used by the editors.
///
#[derive(Debug, Default, Clone, Copy)]
pub struct EditorGroupStyles {
    /// The editors active styles.
    pub active: EditorStyles,
    /// The editors inactive styles.
    pub inactive: EditorStyles,
}

/// The result of some action performed on the editors.
#[derive(Debug, PartialEq)]
pub enum EditorGroupResult {
    /// The action was ignored by the editors.
    Ignored,
    /// The editors performed the action.
    Consumed,
    /// The action was not allowed by the editors.
    NotAllowed,
    /// An editor was not found in the editors group.
    EditorNotFound,
}
/// Convert an [EditorResult] into the corresponding [EditorGroupResult].
impl From<EditorResult> for EditorGroupResult {
    fn from(editor_result: EditorResult) -> Self {
        match editor_result {
            EditorResult::Ignored => EditorGroupResult::Ignored,
            EditorResult::Consumed => EditorGroupResult::Consumed,
            EditorResult::NotAllowed => EditorGroupResult::NotAllowed,
        }
    }
}

/// A collection of editors.
#[derive(Debug)]
pub struct EditorGroup {
    /// The collection of edit fields.
    editors: Vec<Editor>,
    /// Allow the active editor in the collection to move from first to last or vice versa.
    wrap: bool,
    /// The size of the edit group.
    size: Size,
    /// The styles to use when rendering the editor group.
    editor_styles: EditorGroupStyles,
    /// The screen area used when [render](Self::render) was last called.
    screen_area: Rect,
}
impl EditorGroup {
    /// Create a new instance of the editor group.
    ///
    /// # Arguments
    ///
    /// * `editors` is the collection of editors that make up the editor group.
    ///
    pub fn new(mut editors: Vec<Editor>) -> Self {
        debug_assert!(editors.len() > 0, "There are no editors in the EditorGroup");

        // make sure all the readonly editors are not active
        editors.iter_mut().for_each(|editor| {
            if editor.is_readonly() {
                editor.set_active(false);
            }
        });

        // make sure only 1 editor is active in the group
        let active_editors = editors.iter_mut().filter(|editor| editor.is_active()).collect::<Vec<_>>();
        match active_editors.len() {
            1 => (),
            0 => {
                // activate the first editor that is not readonly
                for editor in &mut editors {
                    if !editor.is_readonly() {
                        editor.set_active(true);
                        break;
                    }
                }
            }
            _ => {
                let mut iter = active_editors.into_iter();
                iter.next();
                // all the editors after the first cannot be active
                while let Some(editor) = iter.next() {
                    editor.set_active(false);
                }
            }
        }
        let width = editors.iter().map(|e| e.width()).max().unwrap_or(0);
        let height = editors.len() as u16;
        Self {
            editors,
            size: Size { width, height },
            wrap: false,
            editor_styles: EditorGroupStyles::default(),
            screen_area: Rect::default(),
        }
    }

    /// A builder method that adds the styles for active and not active editors.
    ///
    /// # Arguments
    ///
    /// * `styles` determines how the editors will be rendered.
    ///
    pub fn with_styles(mut self, styles: EditorGroupStyles) -> Self {
        self.editor_styles = styles;
        for editor in &mut self.editors {
            match editor.is_active() {
                true => editor.set_styles(styles.active),
                false => editor.set_styles(styles.inactive),
            }
        }
        self
    }

    /// Force the width of the editor labels.
    ///
    /// # Arguments
    ///
    /// * `width` sets the width of all editor labels when they are rendered.
    ///
    pub fn with_label_width(mut self, width: u16) -> Self {
        self.editors.iter_mut().for_each(|editor| editor.set_label_width(width));
        self.size.width = self.editors.iter().map(|editor| editor.width()).max().unwrap_or(0);
        self
    }

    /// Set the alignment of editor labels. If the alignment is [Center](Alignment::Center) or
    /// [Right](Alignment::Right) the label width will automatically be set to the widest editor label.
    ///
    /// # Arguments
    ///
    /// * `alignment` determines the label alignment.
    ///
    pub fn with_label_alignment(mut self, alignment: Alignment) -> Self {
        self.editors.iter_mut().for_each(|editor| editor.set_label_alignment(alignment));
        match alignment {
            Alignment::Left => self,
            _ => {
                let label_width = self.editors.iter().map(|editor| editor.label_width()).max().unwrap_or(0);
                self.with_label_width(label_width)
            }
        }
    }

    /// A builder method that determines wrapping behavior allowing the active edit field to move
    /// from first to last or vice versa.
    ///
    pub fn with_wrap(mut self) -> Self {
        self.wrap = true;
        self
    }

    /// Get the overall size of the editor group.
    ///
    pub fn size(&self) -> Size {
        self.size
    }

    /// Get an editor from the edit group.
    ///
    /// # Arguments
    ///
    /// * `id` is the editor identifier name.
    ///
    pub fn editor(&self, id: impl ToString) -> Option<&Editor> {
        let id = id.to_string();
        self.editors.iter().find_map(|editor| if editor.id() == &id { Some(editor) } else { None })
    }

    /// Set one of the editors in the collection active.
    ///
    /// # Arguments
    ///
    /// * `id` is the editor identifier name.
    ///
    pub fn set_active(&mut self, id: impl ToString) -> EditorGroupResult {
        let id = id.to_string();
        // scan the fields to make sure there is an id match
        match self.editor(id.as_str()) {
            None => {
                // if you didn't find the field then code is AFU
                log::warn!("Did not find editor id '{id}'.");
                EditorGroupResult::EditorNotFound
            }
            Some(editor) => match editor.is_readonly() {
                true => EditorGroupResult::NotAllowed,
                false => {
                    for editor in &mut self.editors {
                        if editor.id() == &id {
                            editor.set_active(true);
                            editor.set_styles(self.editor_styles.active);
                        } else {
                            editor.set_active(false);
                            editor.set_styles(self.editor_styles.inactive);
                        }
                    }
                    EditorGroupResult::Consumed
                }
            },
        }
    }

    /// Try to make the next editor active. If wrap is set, the first editor will be set active when
    /// the last editor is active.
    ///
    fn next_editor(&mut self) -> EditorGroupResult {
        // the lambda prevents borrowing self as mutable more than once
        let set_active = |editor: &mut Editor, active: bool| {
            editor.set_active(active);
            match editor.is_active() {
                true => editor.set_styles(self.editor_styles.active),
                false => editor.set_styles(self.editor_styles.inactive),
            }
        };

        // get all the editors that are NOT readonly
        let mut editors = self.editors.iter_mut().filter(|editor| !editor.is_readonly()).collect::<Vec<_>>();

        if editors.len() < 2 {
            EditorGroupResult::Ignored
        } else if editors.last().unwrap().is_active() {
            match self.wrap {
                false => EditorGroupResult::NotAllowed,
                true => {
                    set_active(editors.last_mut().unwrap(), false);
                    set_active(editors.first_mut().unwrap(), true);
                    EditorGroupResult::Consumed
                }
            }
        } else {
            let mut iter = editors.into_iter();
            while let Some(current_editor) = iter.next() {
                if current_editor.is_active() {
                    set_active(current_editor, false);
                    set_active(iter.next().unwrap(), true);
                    break;
                }
            }
            EditorGroupResult::Consumed
        }
    }

    /// Try to make the previous editor active. If wrap is set, the last editor will be set active when
    /// the first editor is active.
    ///
    fn previous_editor(&mut self) -> EditorGroupResult {
        // the lambda prevents borrowing self as mutable more than once
        let set_active = |editor: &mut Editor, active: bool| {
            editor.set_active(active);
            match editor.is_active() {
                true => editor.set_styles(self.editor_styles.active),
                false => editor.set_styles(self.editor_styles.inactive),
            }
        };

        // get all the editors that are not readonly
        let mut editors = self.editors.iter_mut().filter(|editor| !editor.is_readonly()).collect::<Vec<_>>();

        if editors.len() < 2 {
            EditorGroupResult::Ignored
        } else if editors.first().unwrap().is_active() {
            match self.wrap {
                false => EditorGroupResult::NotAllowed,
                true => {
                    set_active(editors.first_mut().unwrap(), false);
                    set_active(editors.last_mut().unwrap(), true);
                    EditorGroupResult::Consumed
                }
            }
        } else {
            // reverse the editor list
            editors.reverse();
            // consume the editors looking for the next one active
            let mut iter = editors.into_iter();
            while let Some(editor) = iter.next() {
                if editor.is_active() {
                    set_active(editor, false);
                    set_active(iter.next().unwrap(), true);
                    break;
                }
            }
            EditorGroupResult::Consumed
        }
    }

    /// Process a key event changing which editor is active or passing the event to the active editor.
    ///
    /// # Arguments
    ///
    /// * `key_event` is the key event.
    ///
    pub fn key_pressed(&mut self, key_event: KeyEvent) -> EditorGroupResult {
        match (key_event.modifiers, key_event.code) {
            (KeyModifiers::NONE, KeyCode::Tab | KeyCode::Down) => self.next_editor(),
            (KeyModifiers::NONE, KeyCode::Up) => self.previous_editor(),
            (KeyModifiers::SHIFT, KeyCode::BackTab) => self.previous_editor(),
            (KeyModifiers::ALT, KeyCode::Char(ch)) => {
                match self.editor_by_selector(ch).map_or(None, |editor| Some(editor.id().to_string())) {
                    None => EditorGroupResult::EditorNotFound,
                    Some(id) => self.set_active(id),
                }
            }
            _ => match self.editors.iter_mut().find(|editor| editor.is_active()) {
                None => {
                    log::warn!("Did not find an active editor in the editor group.");
                    EditorGroupResult::EditorNotFound
                }
                Some(editor) => {
                    debug_assert!(!editor.is_readonly(), "The editor active editor is readonly!");
                    editor.key_pressed(&key_event).into()
                }
            },
        }
    }

    /// If the mouse event is over an editor, make it active.
    ///
    /// # Arguments
    ///
    /// * `event` is the mouse event.
    ///
    pub fn left_mouse_button(&mut self, event: MouseEvent) -> EditorGroupResult {
        let mut result = EditorGroupResult::Ignored;

        // make sure the click was within the screen area
        if self.screen_area.contains(Position { x: event.column, y: event.row }) {
            // get the editor that was clicked
            let editor_idx = event.row - self.screen_area.y;
            let editor = &self.editors[editor_idx as usize];

            // make sure the editor should be set active
            if !editor.is_active() && !editor.is_readonly() {
                self.set_active(editor.id().to_string());
                result = EditorGroupResult::Consumed;
            }
        }
        result
    }

    /// Draw the editor group in the buffer and return the coordinates of the active editor.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the edit group will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    pub fn render(&mut self, mut area: Rect, buffer: &mut Buffer) -> Option<Position> {
        // capture the screen area used by the group
        area.height = std::cmp::min(area.height, self.editors.len() as u16);
        if area != self.screen_area {
            self.screen_area = area;
        }

        // render the editors
        area.height = 1;
        let mut position = None;
        for idx in 0..self.screen_area.height as usize {
            let editor = &self.editors[idx];
            if let Some(active_position) = editor.render(area, buffer) {
                position.replace(active_position);
            }
            area.y += 1;
        }
        position
    }

    /// A helper method to find the editor associated with a selector. When multiple
    /// editors match the selector:
    ///
    /// * the first editor in the group is returned if there is not an active editor.
    /// * the first editor in the group is returned when the last editor in the group is active.
    /// * otherwise the editor after the active editor in the group is returned.
    ///
    /// # Arguments
    ///
    /// - `ch` is a selector character.
    ///
    fn editor_by_selector(&self, ch: char) -> Option<&Editor> {
        // find all the editors that match the selector
        let selector = ch.to_lowercase().to_string();
        let mut editors = self
            .editors
            .iter()
            .filter(|editor| editor.label_selector().to_lowercase().to_string() == selector)
            .collect::<Vec<_>>();

        match editors.len() {
            0 => {
                log::warn!("No editors are active.");
                None
            }
            1 => Some(editors.pop()?),
            _ => match editors.last()?.is_active() {
                // wrap the selector if the last editor is currently active
                true => editors.into_iter().next(),
                false => {
                    match editors.iter().filter(|editor| editor.is_active()).count() {
                        0 => editors.into_iter().next(),
                        _ => {
                            // select the next editor after the active one
                            let mut iter = editors.into_iter();
                            while let Some(editor) = iter.next() {
                                if editor.is_active() {
                                    break;
                                }
                            }
                            Some(iter.next()?)
                        }
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::label::{Label, LabelStyles};
    use crossterm::event::{KeyEventKind, KeyEventState};
    use ratatui::prelude::{Color, Position, Style};

    #[test]
    fn from() {
        // check none of the editors being active
        let testcase = EditorGroup::new(vec![
            Editor::default().with_label(Label::new("first")).with_width(5),
            Editor::default().with_label(Label::new("second")).with_width(6),
        ]);
        assert_eq!(testcase.size, Size { width: 12, height: 2 });
        assert!(testcase.editors.first().unwrap().is_active());
        assert!(!testcase.editors.last().unwrap().is_active());

        // check not having the first editor active
        let testcase = EditorGroup::new(vec![
            Editor::default().with_label(Label::new("first")),
            Editor::default().with_label(Label::new("second")).with_active(true),
        ]);
        assert_eq!(testcase.size, Size { width: 6, height: 2 });
        assert!(!testcase.editors.first().unwrap().is_active());
        assert!(testcase.editors.last().unwrap().is_active());

        // check when more than one editor is active
        let testcase = EditorGroup::new(vec![
            Editor::default().with_label(Label::new("first")).with_active(true),
            Editor::default().with_label(Label::new("second")).with_active(true),
        ]);
        assert!(testcase.editors.first().unwrap().is_active());
        assert!(!testcase.editors.last().unwrap().is_active());

        // check when more than one editor is active
        let testcase = EditorGroup::new(vec![
            Editor::default().with_label(Label::new("first")).with_readonly(true).with_active(true),
            Editor::default().with_label(Label::new("second")).with_active(true),
        ]);
        assert!(!testcase.editors.first().unwrap().is_active());
        assert!(testcase.editors.last().unwrap().is_active());
    }

    #[test]
    fn label_width() {
        let mut testcase = EditorGroup::new(vec![
            Editor::default().with_label(Label::new("first")),
            Editor::default().with_label(Label::new("second")),
        ]);
        assert_eq!(testcase.editors[0].label_width(), 5);
        assert_eq!(testcase.editors[1].label_width(), 6);

        testcase = testcase.with_label_width(10);
        assert_eq!(testcase.editors[0].label_width(), 10);
        assert_eq!(testcase.editors[1].label_width(), 10);
    }

    #[test]
    fn label_alignment() {
        // default
        let testcase = EditorGroup::new(vec![
            Editor::default().with_label(Label::new("brown")),
            Editor::default().with_label(Label::new("fox")),
        ]);
        assert_eq!(testcase.editors[0].label_width(), 5);
        assert_eq!(testcase.editors[0].label().unwrap().alignment(), Alignment::Left);
        assert_eq!(testcase.editors[1].label_width(), 3);
        assert_eq!(testcase.editors[1].label().unwrap().alignment(), Alignment::Left);

        // aligned left
        let testcase = EditorGroup::new(vec![
            Editor::default().with_label(Label::new("brown")),
            Editor::default().with_label(Label::new("fox")),
        ])
        .with_label_alignment(Alignment::Left);
        assert_eq!(testcase.editors[0].label_width(), 5);
        assert_eq!(testcase.editors[0].label().unwrap().alignment(), Alignment::Left);
        assert_eq!(testcase.editors[1].label_width(), 3);
        assert_eq!(testcase.editors[1].label().unwrap().alignment(), Alignment::Left);

        // aligned center
        let testcase = EditorGroup::new(vec![
            Editor::default().with_label(Label::new("brown")),
            Editor::default().with_label(Label::new("fox")),
        ])
        .with_label_alignment(Alignment::Center);
        assert_eq!(testcase.editors[0].label_width(), 5);
        assert_eq!(testcase.editors[0].label().unwrap().alignment(), Alignment::Center);
        assert_eq!(testcase.editors[1].label_width(), 5);
        assert_eq!(testcase.editors[1].label().unwrap().alignment(), Alignment::Center);

        // aligned right
        let testcase = EditorGroup::new(vec![
            Editor::default().with_label(Label::new("brown")),
            Editor::default().with_label(Label::new("fox")),
        ])
        .with_label_alignment(Alignment::Right);
        assert_eq!(testcase.editors[0].label_width(), 5);
        assert_eq!(testcase.editors[0].label().unwrap().alignment(), Alignment::Right);
        assert_eq!(testcase.editors[1].label_width(), 5);
        assert_eq!(testcase.editors[1].label().unwrap().alignment(), Alignment::Right);
    }

    #[test]
    fn active() {
        // verify the defaults
        let mut testcase = EditorGroup::new(vec![
            Editor::default().with_id("one").with_active(true),
            Editor::default().with_id("two"),
            Editor::default().with_id("three").with_readonly(true),
        ]);
        assert!(testcase.editors[0].is_active());
        assert!(!testcase.editors[1].is_active());
        assert!(!testcase.editors[2].is_active());

        // try to set active an id that will not be found
        assert_eq!(testcase.set_active("foobar"), EditorGroupResult::EditorNotFound);

        // set active the 2nd editor
        assert_eq!(testcase.set_active("two"), EditorGroupResult::Consumed);
        assert!(!testcase.editors[0].is_active());
        assert!(testcase.editors[1].is_active());
        assert!(!testcase.editors[2].is_active());

        // try to set the readonly editor active
        assert_eq!(testcase.set_active("three"), EditorGroupResult::NotAllowed);
        assert!(!testcase.editors[0].is_active());
        assert!(testcase.editors[1].is_active());
        assert!(!testcase.editors[2].is_active());
    }

    #[test]
    fn next_editor() {
        let mut testcase = EditorGroup::new(vec![
            Editor::default().with_label(Label::new("one")).with_active(true),
            Editor::default().with_label(Label::new("two")),
            Editor::default().with_label(Label::new("three")),
            Editor::default().with_label(Label::new("four")).with_readonly(true),
        ]);
        assert_eq!(testcase.next_editor(), EditorGroupResult::Consumed);
        assert!(!testcase.editors[0].is_active());
        assert!(testcase.editors[1].is_active());
        assert_eq!(testcase.next_editor(), EditorGroupResult::Consumed);
        assert!(!testcase.editors[1].is_active());
        assert!(testcase.editors[2].is_active());
        assert_eq!(testcase.next_editor(), EditorGroupResult::NotAllowed);
        assert!(!testcase.editors[1].is_active());
        assert!(testcase.editors[2].is_active());

        // turn on wrapping
        testcase = testcase.with_wrap();
        assert_eq!(testcase.next_editor(), EditorGroupResult::Consumed);
        assert!(!testcase.editors[2].is_active());
        assert!(testcase.editors[0].is_active());
    }

    #[test]
    fn prev_editor() {
        let mut testcase = EditorGroup::new(vec![
            Editor::default().with_label(Label::new("one")),
            Editor::default().with_label(Label::new("two")),
            Editor::default().with_label(Label::new("three")).with_active(true),
        ]);
        assert_eq!(testcase.previous_editor(), EditorGroupResult::Consumed);
        assert!(!testcase.editors[2].is_active());
        assert!(testcase.editors[1].is_active());
        assert_eq!(testcase.previous_editor(), EditorGroupResult::Consumed);
        assert!(!testcase.editors[1].is_active());
        assert!(testcase.editors[0].is_active());
        assert_eq!(testcase.previous_editor(), EditorGroupResult::NotAllowed);
        assert!(!testcase.editors[1].is_active());
        assert!(testcase.editors[0].is_active());

        // turn on wrapping
        testcase = testcase.with_wrap();
        assert_eq!(testcase.previous_editor(), EditorGroupResult::Consumed);
        assert!(!testcase.editors[0].is_active());
        assert!(testcase.editors[2].is_active());
    }

    #[test]
    fn editor_by_selector() {
        let mut testcase = EditorGroup::new(vec![
            Editor::default().with_id("one").with_label(Label::new("o").with_selector('o')),
            Editor::default().with_id("two").with_label(Label::new("t").with_selector('t')),
            Editor::default().with_id("three").with_label(Label::new("t").with_selector('t')),
        ]);
        assert!(testcase.editor_by_selector('a').is_none());
        assert_eq!(testcase.editor_by_selector('o').unwrap().id(), "one");
        assert_eq!(testcase.editor_by_selector('t').unwrap().id(), "two");
        assert_eq!(testcase.editor_by_selector('t').unwrap().id(), "two");
        testcase.set_active("two");
        assert_eq!(testcase.editor_by_selector('t').unwrap().id(), "three");
        testcase.set_active("three");
        assert_eq!(testcase.editor_by_selector('t').unwrap().id(), "two");
    }

    #[test]
    fn styles() {
        let styles = EditorGroupStyles {
            active: EditorStyles {
                text: Style::default().light_red(),
                label: LabelStyles { text: Style::default().light_green(), selector: Style::default().light_blue() },
            },
            inactive: EditorStyles {
                text: Style::default().red(),
                label: LabelStyles { text: Style::default().green(), selector: Style::default().blue() },
            },
        };
        let mut testcase = EditorGroup::new(vec![
            Editor::new("one").with_id("o").with_active(true).with_label(Label::new("one")),
            Editor::new("two").with_id("t").with_label(Label::new("two")),
        ])
        .with_styles(styles);
        assert_eq!(testcase.editors[0].styles(), styles.active);
        assert_eq!(testcase.editors[1].styles(), styles.inactive);

        assert_eq!(testcase.set_active("t"), EditorGroupResult::Consumed);
        assert_eq!(testcase.editors[0].styles(), styles.inactive);
        assert_eq!(testcase.editors[1].styles(), styles.active);
    }

    #[test]
    fn set_active() {
        let mut testcase = EditorGroup::new(vec![
            Editor::default().with_id("one").with_active(true),
            Editor::default().with_id("two").with_readonly(true),
            Editor::default().with_id("three"),
        ]);
        assert_eq!(testcase.set_active("foobar"), EditorGroupResult::EditorNotFound);
        assert_eq!(testcase.set_active("two"), EditorGroupResult::NotAllowed);
        assert_eq!(testcase.set_active("three"), EditorGroupResult::Consumed);
        assert!(!testcase.editors[0].is_active());
        assert!(testcase.editors[2].is_active());
    }

    #[test]
    fn key_press() {
        let mut testcase = EditorGroup::new(vec![
            Editor::default().with_id("one").with_label(Label::new("one").with_selector('o')).with_active(true),
            Editor::default().with_id("two").with_label(Label::new("two").with_selector('t')),
        ])
        .with_wrap();

        macro_rules! key_event {
            ($code: expr, $modifier: expr) => {
                KeyEvent { code: $code, modifiers: $modifier, kind: KeyEventKind::Press, state: KeyEventState::empty() }
            };
            ($code: expr) => {
                KeyEvent {
                    code: $code,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                }
            };
        }

        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Tab)), EditorGroupResult::Consumed);
        assert!(!testcase.editors[0].is_active());
        assert!(testcase.editors[1].is_active());
        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Tab)), EditorGroupResult::Consumed);
        assert!(testcase.editors[0].is_active());
        assert!(!testcase.editors[1].is_active());

        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Down)), EditorGroupResult::Consumed);
        assert!(!testcase.editors[0].is_active());
        assert!(testcase.editors[1].is_active());
        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Down)), EditorGroupResult::Consumed);
        assert!(testcase.editors[0].is_active());
        assert!(!testcase.editors[1].is_active());

        assert_eq!(
            testcase.key_pressed(key_event!(KeyCode::BackTab, KeyModifiers::SHIFT)),
            EditorGroupResult::Consumed
        );
        assert!(!testcase.editors[0].is_active());
        assert!(testcase.editors[1].is_active());
        assert_eq!(
            testcase.key_pressed(key_event!(KeyCode::BackTab, KeyModifiers::SHIFT)),
            EditorGroupResult::Consumed
        );
        assert!(testcase.editors[0].is_active());
        assert!(!testcase.editors[1].is_active());

        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Up)), EditorGroupResult::Consumed);
        assert!(!testcase.editors[0].is_active());
        assert!(testcase.editors[1].is_active());
        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Up)), EditorGroupResult::Consumed);
        assert!(testcase.editors[0].is_active());
        assert!(!testcase.editors[1].is_active());

        assert_eq!(
            testcase.key_pressed(key_event!(KeyCode::Char('t'), KeyModifiers::ALT)),
            EditorGroupResult::Consumed
        );
        assert!(!testcase.editors[0].is_active());
        assert!(testcase.editors[1].is_active());
        assert_eq!(
            testcase.key_pressed(key_event!(KeyCode::Char('a'), KeyModifiers::ALT)),
            EditorGroupResult::EditorNotFound
        );
        assert!(!testcase.editors[0].is_active());
        assert!(testcase.editors[1].is_active());
        assert_eq!(
            testcase.key_pressed(key_event!(KeyCode::Char('o'), KeyModifiers::ALT)),
            EditorGroupResult::Consumed
        );
        assert!(testcase.editors[0].is_active());
        assert!(!testcase.editors[1].is_active());

        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Char('o'))), EditorGroupResult::Consumed);
        assert_eq!(testcase.editors[0].text(), "o");
        assert_eq!(testcase.editors[1].text(), "");

        testcase.set_active("two");
        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Char('t'))), EditorGroupResult::Consumed);
        assert_eq!(testcase.editors[0].text(), "o");
        assert_eq!(testcase.editors[1].text(), "t");
    }

    #[test]
    fn render() {
        let area = Rect::new(0, 0, 10, 2);
        let mut buffer = Buffer::empty(area);
        macro_rules! assert_cell {
            ($x: literal, $y: literal, $value:expr, $style:expr) => {
                assert_eq!(buffer[($x, $y)].symbol(), $value);
                assert_eq!(buffer[($x, $y)].style(), $style);
            };
        }
        let default_cell = &buffer[(0, 0)].clone();
        let styles = EditorGroupStyles {
            active: EditorStyles {
                text: Style::default().red().bg(Color::Reset).underline_color(Color::Reset),
                label: LabelStyles {
                    text: Style::default().green().bg(Color::Reset).underline_color(Color::Reset),
                    selector: Style::default().blue().bg(Color::Reset).underline_color(Color::Reset),
                },
            },
            inactive: EditorStyles {
                text: Style::default().light_red().bg(Color::Reset).underline_color(Color::Reset),
                label: LabelStyles {
                    text: Style::default().light_green().bg(Color::Reset).underline_color(Color::Reset),
                    selector: Style::default().light_blue().bg(Color::Reset).underline_color(Color::Reset),
                },
            },
        };
        let mut testcase = EditorGroup::new(vec![
            Editor::new("one").with_id("one").with_label(Label::new("one:").with_selector('o')).with_active(true),
            Editor::new("two").with_id("two").with_label(Label::new("two:").with_selector('t')),
        ])
        .with_label_width(5)
        .with_styles(styles);

        // the initial render
        assert_eq!(testcase.render(area, &mut buffer).unwrap(), Position { x: 8, y: 0 });
        assert_cell!(0, 0, "o", styles.active.label.selector);
        assert_cell!(1, 0, "n", styles.active.label.text);
        assert_cell!(2, 0, "e", styles.active.label.text);
        assert_cell!(3, 0, ":", styles.active.label.text);
        assert_cell!(4, 0, " ", default_cell.style());
        assert_cell!(5, 0, "o", styles.active.text);
        assert_cell!(6, 0, "n", styles.active.text);
        assert_cell!(7, 0, "e", styles.active.text);
        assert_cell!(0, 1, "t", styles.inactive.label.selector);
        assert_cell!(1, 1, "w", styles.inactive.label.text);
        assert_cell!(2, 1, "o", styles.inactive.label.text);
        assert_cell!(3, 1, ":", styles.inactive.label.text);
        assert_cell!(4, 1, " ", default_cell.style());
        assert_cell!(5, 1, "t", styles.inactive.text);
        assert_cell!(6, 1, "w", styles.inactive.text);
        assert_cell!(7, 1, "o", styles.inactive.text);
    }
}
