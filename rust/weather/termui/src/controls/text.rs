//! The TUI text edit controls.
//!
//! An [EditField] is a modifiable text box. All `EditField` controls contain
//! a left hand side [Label] and a right hand side [Editor]. All `EditField`
//! controls implement the [FieldEditor] trait.
//!
//! A [EditFieldGroup] control is available to manage a vertical group of
//! `EditField` controls.
//!

pub use date_editor::DateEditor;
mod date_editor;

pub use text_editor::TextEditor;
mod text_editor;

use super::*;
use {
    label::Label,
    styles::{ActiveNormalStyles, CatalogType, StyleCatalog, StyleId},
};

/// The internal API editors must support.
trait FieldEditor {
    /// Get the editor screen size.
    fn size(&self) -> Size;
    /// Dispatch a key pressed event to the editor and return the result.
    ///
    /// # Arguments
    ///
    /// * `key_event` is a key pressed event.
    ///
    fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult>;
    /// Draw the editor on the screen and return the current cursor coordinate.
    ///
    /// # Arguments
    ///
    /// * `area` is where on the screen the editor will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// * `styles` is catalog that will be used to render the editor.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer, styles: &StyleCatalog) -> Position;
    /// Return the editor text.
    ///
    fn text(&self) -> &str;
    /// Set the editor text.
    ///
    /// # Arguments
    ///
    /// * `content` is the text set into the editor.
    ///
    /// todo: This needs to return a Result in case the text being set is not correct.
    ///
    fn set_text(&mut self, text: impl ToString);
}

/// The current text editors.
#[derive(Debug)]
pub enum Editor {
    Date(DateEditor),
    Text(TextEditor),
}
impl std::fmt::Display for Editor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let editor = match self {
            Editor::Date(_) => "Date",
            Editor::Text(_) => "Text",
        };
        write!(f, "Editor({editor})")
    }
}
impl From<DateEditor> for Editor {
    /// Convert the date iterator into an editor.
    fn from(editor: DateEditor) -> Self {
        Editor::Date(editor)
    }
}
impl From<TextEditor> for Editor {
    /// Convert the text editor into an editor.
    fn from(editor: TextEditor) -> Self {
        Editor::Text(editor)
    }
}
impl FieldEditor for Editor {
    /// Get the size of the editor.
    fn size(&self) -> Size {
        match self {
            Editor::Date(date) => date.size(),
            Editor::Text(text) => text.size(),
        }
    }
    /// Dispatch a key pressed event to the editor.
    ///
    /// # Arguments
    ///
    /// * `key_event` is a key pressed event.
    ///
    fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
        match self {
            Editor::Date(date) => date.key_pressed(key_event),
            Editor::Text(text) => text.key_pressed(key_event),
        }
    }
    /// Render the editor and return the current cursor coordinate.
    ///
    /// # Arguments
    ///
    /// * `area` is where on the screen the editor will be rendered.
    /// * `buffer` is where the rendering is sent.
    /// * `styles` is catalog that will be used to render the editor.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer, styles: &StyleCatalog) -> Position {
        match self {
            Editor::Date(date) => date.render(area, buffer, styles),
            Editor::Text(text) => text.render(area, buffer, styles),
        }
    }
    /// Return the editor text.
    fn text(&self) -> &str {
        match self {
            Editor::Date(date) => date.text(),
            Editor::Text(text) => text.text(),
        }
    }
    /// Set the editor text.
    ///
    /// # Arguments
    ///
    /// * `content` is the text set into the editor.
    ///
    fn set_text(&mut self, content: impl ToString) {
        match self {
            Editor::Date(date) => date.set_text(content),
            Editor::Text(text) => text.set_text(content),
        }
    }
}

/// The text field editor.
#[derive(Debug)]
pub struct EditField {
    /// A description of the text being edited.
    label: Label,
    /// The editor that will be used.
    editor: Editor,
}
impl std::fmt::Display for EditField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EditField[{}] editor={} active={}", self.label.id(), self.editor, self.label.is_active())
    }
}
impl EditField {
    /// Create a new instance of the field editor.
    ///
    /// # Arguments
    ///
    /// * `label` is used to describe the edit field.
    /// * `editor` is the type of editor that will be used.
    ///
    pub fn new(label: Label, editor: impl Into<Editor>) -> Self {
        Self { label, editor: editor.into() }
    }
}
impl Control for EditField {
    /// Get the control identifier.
    fn id(&self) -> &str {
        self.label.id()
    }
    /// Get the edit field selector character.
    ///
    fn selector(&self) -> char {
        self.label.selector()
    }
    /// Get the size of the edit field.
    ///
    fn size(&self) -> Size {
        let mut size = self.label.size();
        let editor_size = self.editor.size();
        size.width += editor_size.width;
        size.height = cmp::max(size.height, editor_size.height);
        size
    }
    /// Query if the edit field is active.
    ///
    fn is_active(&self) -> bool {
        self.label.is_active()
    }
    /// Set the active state of the edit field.
    ///
    /// # Arguments
    ///
    /// - `active` is the edit field state.
    ///
    fn set_active(&mut self, active: bool) {
        self.label.set_active(active)
    }
    /// Draw the edit field on the terminal screen and return the coordinate of its selector if active.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the button will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` contains the [styles](StyleCatalog) used the draw the button.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer, styles: &StyleCatalog) -> Option<Position> {
        log_render!(self.to_string());
        // show the label
        let label_size = self.label.size();
        let label_area = inner_rect(area, (0, 0), (label_size.width as i32, label_size.height as i32));
        self.label.render(label_area, buffer, styles);
        // show the editor
        let text_area = inner_rect(area, (label_size.width as i32, 0), (0, label_size.height as i32));
        let editor_coord = self.editor.render(text_area, buffer, styles);
        match self.label.is_active() {
            true => Some(editor_coord),
            false => None,
        }
    }
    /// Allow the editor to process a key pressed event. It will return [ControlFlow::Continue]
    /// if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key press](KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!(self.to_string());
        self.editor.key_pressed(key_event)
    }
}
impl EditControl for EditField {
    /// Get the contents of the edit field.
    fn text(&self) -> &str {
        self.editor.text()
    }
    /// Set the contents of the edit field.
    ///
    /// # Arguments
    ///
    /// - `text` will replace the previous content of the edit field.
    ///
    fn set_text(&mut self, text: impl ToString) {
        self.editor.set_text(text);
    }
}

/// A managed collection of edit fields.
#[derive(Debug)]
pub struct EditFieldGroup {
    /// Track if the edit group is active or not.
    pub active: bool,
    /// The collection of edit fields.
    fields: Vec<EditField>,
    /// The optional title of the edit field group.
    title: Option<String>,
    /// The alignment of the title.
    title_alignment: Alignment,
    /// Allow the active edit field in the collection to move from first to last or vice versa.
    wrap: bool,
    /// Center the edit field descriptions instead of being left justified.
    center_fields: bool,
    /// The size of the edit group.
    size: Size,
    /// The checkbox group style catalog type. This will always be [CatalogType::EditGroup].
    pub catalog_type: CatalogType,
}
impl EditFieldGroup {
    /// Create a new instance of the edit field group.
    ///
    /// # Arguments
    ///
    /// = `fields` is the collection of edit field that will be managed.
    ///
    pub fn new(fields: Vec<EditField>) -> Self {
        debug_assert!(fields.len() > 0);
        let size =
            Size { width: fields.iter().map(|f| f.size().width).max().unwrap_or(0), height: fields.len() as u16 };
        Self {
            active: false,
            fields,
            title: Default::default(),
            title_alignment: Alignment::Center,
            wrap: false,
            center_fields: false,
            size,
            catalog_type: CatalogType::EditGroup,
        }
    }
    /// A builder method that forces labels to be aligned.
    ///
    pub fn with_labels_aligned(mut self) -> Self {
        let max_width = self.fields.iter().map(|f| f.label.size().width).max().unwrap();
        self.fields = self
            .fields
            .into_iter()
            .map(|mut edit_field| {
                edit_field.label = edit_field.label.with_width(max_width);
                edit_field
            })
            .collect();
        self
    }
    /// A builder method that forces centered labels.
    ///
    pub fn with_centered_fields(mut self) -> Self {
        self.center_fields = true;
        self
    }
    /// A builder method that sets the title of an edit group.
    ///
    /// # Arguments
    ///
    /// - `title` is the edit group title.
    ///
    pub fn with_title(mut self, title: impl ToString) -> Self {
        let title = title.to_string();
        self.size.width = cmp::max(self.size.width, title.len() as u16);
        self.size.height += 1;
        self.title = Some(title.to_string());
        self
    }
    /// A builder method that sets the alignment of the group title.
    ///
    /// # Arguments
    ///
    /// - `title_alignment` is the alignment that will be used for the title.
    ///
    pub fn with_title_alignment(mut self, title_alignment: Alignment) -> Self {
        self.title_alignment = title_alignment;
        self
    }
    /// A builder method that sets edit field wrapping behaviour allowing the active edit field to move
    /// from first to last or vice versa.
    ///
    pub fn with_wrap(mut self) -> Self {
        self.wrap = true;
        self
    }
    /// A builder method that sets the initial state of the edit group active.
    ///
    pub fn with_active(mut self) -> Self {
        self.active = true;
        self
    }
    /// A helper method that scans the edit fields for a selector character match.
    ///
    /// # Arguments
    ///
    /// - `ch` is some edit field selector character.
    ///
    fn find_selector(&self, ch: char) -> Option<&EditField> {
        let lhs = ch.to_lowercase().to_string();
        self.fields.iter().find(|edit_field| lhs == edit_field.selector().to_lowercase().to_string())
    }
}
impl ControlGroup<EditField> for EditFieldGroup {
    /// Get the size of the control group.
    ///
    fn size(&self) -> Size {
        self.size
    }
    /// Get an edit field from the group.
    ///
    /// # Arguments
    ///
    /// - `id` is the edit field identifier.
    ///
    fn get(&self, id: impl ToString) -> Option<&EditField> {
        let id = id.to_string();
        self.fields.iter().find_map(|edit_field| if edit_field.id() == &id { Some(edit_field) } else { None })
    }
    /// Get a mutable edit field from the group.
    ///
    /// # Arguments
    ///
    /// - `id` is the edit field identifier.
    ///
    fn get_mut(&mut self, id: impl ToString) -> Option<&mut EditField> {
        let id = id.to_string();
        self.fields.iter_mut().find_map(|edit_field| if edit_field.id() == &id { Some(edit_field) } else { None })
    }
    /// Set an edit field within the group active.
    ///
    /// # Arguments
    ///
    /// - `id` is the edit field identifier.
    ///
    fn set_active(&mut self, id: impl ToString) -> ControlFlow<ControlResult> {
        let id = id.to_string();
        // scan the fields to make sure there is an id match
        match self.fields.iter().find(|edit_field| &id == edit_field.id()) {
            None => {
                // if you didn't find the field then code is AFU
                debug_assert!(false, "Did not find edit field '{}' in the edit group\n{:#?}", id, self);
            }
            Some(_) => {
                self.fields.iter_mut().for_each(|edit_field| edit_field.set_active(&id == edit_field.id()));
            }
        }
        break_event!(ControlResult::Continue)
    }
    /// Pass a key pressed event to edit fields in the group. [ControlFlow::Continue] is returned if there
    /// was not an edit field that consumed the event.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!("EditFieldGroup");
        match (key_event.modifiers, key_event.code) {
            (KeyModifiers::NONE, KeyCode::Tab | KeyCode::Down) => {
                match next_control(as_mut_refs!(self.fields), self.wrap) {
                    true => break_event!(ControlResult::Continue)?,
                    false => break_event!(ControlResult::NextGroup)?,
                }
            }
            (KeyModifiers::NONE, KeyCode::Up) => {
                match previous_control(as_mut_refs!(self.fields), self.wrap) {
                    true => break_event!(ControlResult::Continue)?,
                    false => break_event!(ControlResult::PrevGroup)?,
                }
            }
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                match previous_control(as_mut_refs!(self.fields), self.wrap) {
                    true => break_event!(ControlResult::Continue)?,
                    false => break_event!(ControlResult::PrevGroup)?,
                }
            }
            (KeyModifiers::ALT, KeyCode::Char(ch)) => match self.find_selector(ch) {
                // continue and don't break to allow the ALT key to be seen by other controls
                None => (),
                Some(edit_field) => {
                    let id = edit_field.id().to_string();
                    self.set_active(id)?
                }
            },
            _ => {
                for edit_field in &mut self.fields {
                    if edit_field.is_active() {
                        edit_field.key_pressed(&key_event)?;
                    }
                }
                // continue and don't break to allow the key to be seen by other controls
            }
        }
        ControlFlow::Continue(())
    }
    /// Get a reference to the controls in the group.
    ///
    fn controls(&self) -> Vec<&EditField> {
        self.fields.iter().map(|ef| ef).collect()
    }
    /// Draw the edit field group on the terminal screen and return the coordinate of the active edit field selector.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the edit group will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` contains the [styles](ActiveNormalStyles) used the draw the edit field group.
    ///
    fn render(&self, mut area: Rect, buffer: &mut Buffer, styles: ActiveNormalStyles) -> Option<Position> {
        log_render!("EditFieldGroup");
        // show the title if one has been set
        if let Some(title) = &self.title {
            let title_style = match self.active {
                true => styles.active.get(StyleId::GroupTitle),
                false => styles.normal.get(StyleId::GroupTitle),
            };
            Block::new()
                .title(Line::from(title.as_str()).style(title_style))
                .title_alignment(self.title_alignment)
                .borders(Borders::NONE)
                .render(area, buffer);
            area = inner_rect(area, (1, 1), (-1, 0));
        }
        // check to see if the controls should be centered in the area
        if self.center_fields {
            let fields_width = self.fields.iter().map(|f| f.size().width).max().unwrap_or(0);
            let fields_height = self.fields.len() as u16;
            area = center_rect!(area, [fields_width, fields_height]);
        }
        // show the edit fields
        let visible_height = cmp::min(area.height as usize, self.fields.len());
        area.height = 1;
        let mut coord = None;
        for idx in 0..visible_height {
            let edit_field = &self.fields[idx];
            let catalog = if edit_field.is_active() { styles.active } else { styles.normal };
            if let Some(field_coord) = edit_field.render(area, buffer, catalog) {
                coord.replace(field_coord);
            }
            area.y += 1;
        }
        coord
    }
    /// Resets all edit fields to a not active state.
    ///
    fn clear_active(&mut self) {
        for field in self.fields.iter_mut() {
            field.set_active(false);
        }
    }
    /// Set the first edit field in the group to an active state
    ///
    fn set_first_active(&mut self) {
        self.fields.first_mut().unwrap().set_active(true);
        self.fields[1..].iter_mut().for_each(|field| field.set_active(false));
    }
    /// Set the last edit field in the group to an active state
    ///
    fn set_last_active(&mut self) {
        self.fields.last_mut().unwrap().set_active(true);
        let len = self.fields.len() - 1;
        self.fields[..len].iter_mut().for_each(|field| field.set_active(false));
    }
}
