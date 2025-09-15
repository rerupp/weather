//! The terminal UI checkbox controls.
//!
//! The [Checkbox] allows selection of an option. A `Checkbox` always has an identifier,
//! description, and selector key. A `Checkbox` always has an identifier, label, and
//! selector key.
//!
//! The `Checkbox` can be selected using an `ALT-key` sequence or `key` press where `key`
//! matches the selector key. It can also be configured to select when active
//! and the `Enter` key is pressed.
//!
use super::*;
use ratatui::widgets::block::Title;
use styles::{CatalogType, StyleCatalog, StyleId};

/// A checkbox will always be this wide.
const CHECKMARK_WIDTH: u16 = 4;

/// The checkbox data structure.
#[derive(Debug)]
pub struct Checkbox {
    /// The checkbox label.
    label: Label,
    /// Indicates the checkbox is selected.
    checked: bool,
}
impl std::fmt::Display for Checkbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CheckBox[{}] active={} checked={}", self.label.id(), self.label.is_active(), self.checked)
    }
}
impl Checkbox {
    /// Create a new checkbox with the required metadata (the checkbox is not checked by default).
    ///
    /// # Arguments
    ///
    /// - `id` is the checkbox identifier.
    /// - `description` is the checkbox description.
    /// - `selector` is a character in the description that can be used to select the checkbox.
    ///
    pub fn new(id: impl ToString, description: impl ToString, selector: char) -> Self {
        Self { label: Label::align_left(description).with_id(id).with_selector(selector), checked: false }
    }
    /// Find out if the checkbox is checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }
    /// Set the checked state of the checkbox.
    ///
    /// # Arguments
    ///
    /// - `yes_no` indicates if the checkbox appears checked or not.
    ///
    pub fn set_checked(&mut self, yes_no: bool) {
        self.checked = yes_no;
    }
}
impl Control for Checkbox {
    /// Get the checkbox identifier attribute.
    ///
    fn id(&self) -> &str {
        self.label.id()
    }
    /// Get the checkbox selection character attribute.
    ///
    fn selector(&self) -> char {
        self.label.selector()
    }
    /// Get the size of the checkbox.
    ///
    fn size(&self) -> Size {
        // the width needs to take into account the checkbox text
        let mut size = self.label.size();
        size.width += CHECKMARK_WIDTH;
        size
    }
    /// Find out if the checkbox is active or not.
    ///
    fn is_active(&self) -> bool {
        self.label.is_active()
    }
    /// Set the checkbox active state.
    ///
    /// # Arguments
    ///
    /// - `active` is the checkbox active state.
    ///
    fn set_active(&mut self, active: bool) {
        self.label.set_active(active);
    }
    /// Draw the checkbox on the terminal screen.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the checkbox will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `catalog` contains the [styles](StyleCatalog) used the draw the checkbox.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer, catalog: &StyleCatalog) -> Option<Position> {
        log_render!(self.to_string());
        match area.height == 0 {
            true => None,
            false => {
                // show the checked indicator
                let checkmark = format!("[{}]", if self.checked { '\u{25fc}' } else { ' ' });
                let checkmark_area = inner_rect(area, (0, 0), (CHECKMARK_WIDTH as i32, 0));
                Paragraph::new(Line::from(checkmark)).alignment(Alignment::Left).render(checkmark_area, buffer);
                // show the label
                let label_area = inner_rect(area, (CHECKMARK_WIDTH as i32, 0), (0, 0));
                match self.label.render(label_area, buffer, catalog).is_some() {
                    true => Some(Position::new(checkmark_area.x + 1, checkmark_area.y)),
                    false => None,
                }
            }
        }
    }
    /// Consume a key pressed event. The checkbox will return [Continue](ControlFlow::Continue) if the event was
    /// not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a key pressed event.
    ///
    fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!(self.to_string());
        // for this control make sure only a key press will select or deselects the checkbox
        match (key_event.modifiers, key_event.code) {
            (KeyModifiers::NONE, KeyCode::Char(' ')) => {
                self.checked = !self.checked;
                break_event!(ControlResult::Continue)
            }
            _ => self.label.key_pressed(key_event),
        }
    }
}

/// The managed collection of [Checkbox] controls.
#[derive(Debug)]
pub struct CheckBoxGroup {
    /// Used to indicate the collection of checkbox controls is active.
    pub active: bool,
    /// The collection of checkbox controls.
    fields: Vec<Checkbox>,
    /// An optional description of the control group.
    title: Option<String>,
    /// Indicates if the description should appear on the left, center, or right of the checkbox controls
    title_alignment: Alignment,
    /// Allow the active checkbox in the collection to move from first to last or vice versa.
    wrap: bool,
    /// Center the field descriptions instead of being left justified.
    center_fields: bool,
    /// The checkbox group style catalog type. This will always be [CatalogType::CheckBoxGroup].
    pub catalog_type: CatalogType,
}
impl CheckBoxGroup {
    /// Creates the managed group of checkbox controls. The group will not have a title, controls will not wrap,
    /// and field descriptions are not aligned.
    ///
    /// # Arguments
    ///
    /// - `checkboxes` is the collection of checkbox controls that will be managed.
    ///
    pub fn new(checkboxes: Vec<Checkbox>) -> Self {
        debug_assert!(checkboxes.len() > 1);
        Self {
            active: false,
            fields: checkboxes,
            title: None,
            title_alignment: Alignment::Center,
            wrap: false,
            center_fields: false,
            catalog_type: CatalogType::CheckBoxGroup,
        }
    }
    /// A builder method that forces all checkbox controls to have aligned labels.
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
    /// A builder method to set if controls in the group should wrap or not.
    ///
    /// # Arguments
    ///
    /// - `yes_no` indicates if the controls should wrap or not.
    ///
    pub fn with_wrap(mut self) -> Self {
        self.wrap = true;
        self
    }
    /// A builder method that sets a title description for the checkbox group.
    ///
    /// # Arguments
    ///
    /// - `title` is the title description that will be used.
    ///
    pub fn with_title(mut self, title: impl ToString) -> Self {
        self.title = Some(title.to_string());
        self
    }
    /// A builder method that indicates the alignment of the checkbox group title.
    ///
    /// # Arguments
    ///
    /// - `alignment` controls where the title is positioned relative to the checkbox controls.
    ///
    pub fn with_title_alignment(mut self, alignment: Alignment) -> Self {
        self.title_alignment = alignment;
        self
    }
    /// A builder method that forces the checkbox labels to be aligned.
    ///
    pub fn with_centered_fields(mut self) -> Self {
        self.center_fields = true;
        self
    }
    /// A helper method that searches the checkbox controls for one that matches the selector character.
    ///
    /// # Arguments
    ///
    /// - `selector` is the checkbox selector that will be matched.
    ///
    fn find_selector(&self, selector: char) -> Option<&Checkbox> {
        let lhs = selector.to_lowercase().to_string();
        self.fields.iter().find(|edit_field| lhs == edit_field.selector().to_lowercase().to_string())
    }
}
impl ControlGroup<Checkbox> for CheckBoxGroup {
    /// Get the size of the checkbox group.
    ///
    fn size(&self) -> Size {
        let width = self.fields.iter().map(|f| f.size().width).max().unwrap_or(0);
        let mut height = self.fields.len() as u16;
        if self.title.is_some() {
            height += 1;
        }
        Size { width, height }
    }
    /// Get a checkbox from the group.
    ///
    /// # Arguments
    ///
    /// - `id` is the checkbox identifier.
    ///
    fn get(&self, id: impl ToString) -> Option<&Checkbox> {
        let id = id.to_string();
        self.fields.iter().find_map(|edit_field| if edit_field.id() == &id { Some(edit_field) } else { None })
    }
    /// Get a mutable checkbox from the group.
    ///
    /// # Arguments
    ///
    /// - `id` is the checkbox identifier.
    ///
    fn get_mut(&mut self, id: impl ToString) -> Option<&mut Checkbox> {
        let id = id.to_string();
        self.fields.iter_mut().find_map(|edit_field| if edit_field.id() == &id { Some(edit_field) } else { None })
    }
    /// Set a checkbox active state to true.
    ///
    /// # Arguments
    ///
    /// - `id` is the checkbox identifier.
    ///
    fn set_active(&mut self, id: impl ToString) -> ControlFlow<ControlResult> {
        let id = id.to_string();
        // scan the fields to make sure there is an id match
        match self.fields.iter().find(|edit_field| &id == edit_field.id()) {
            // None => break_event!(ControlResult::KeyMunched(false)),
            None => break_event!(ControlResult::NotAllowed),
            Some(_) => {
                self.fields.iter_mut().for_each(|edit_field| edit_field.set_active(&id == edit_field.id()));
                // break_event!(ControlResult::KeyMunched(true))
                break_event!(ControlResult::Continue)
            }
        }
    }
    /// Consume a key pressed event. The checkbox group will return [Continue](ControlFlow::Continue) if the event
    /// was not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a key pressed event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!("CheckBoxGroup");
        match (key_event.modifiers, key_event.code) {
            (KeyModifiers::NONE, KeyCode::Tab) => {
                // next_control returns false if the last checkbox is active and wrap false otherwise true
                match next_control(as_mut_refs!(self.fields), self.wrap) {
                    true => break_event!(ControlResult::Continue)?,
                    false => break_event!(ControlResult::NextGroup)?,
                }
            }
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                // previous_control returns false if the first checkbox is active and wrap false otherwise true
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
                match self.fields.iter_mut().find(|checkbox| checkbox.is_active()).take() {
                    None => (),
                    Some(checkbox) => {
                        let result = checkbox.key_pressed(&key_event);
                        if break_event!(ControlResult::Continue) == result {
                            next_control(as_mut_refs!(self.fields), false);
                        }
                        result?;
                    }
                }
                // continue and don't break to allow the key to be seen by other controls
            }
        }
        ControlFlow::Continue(())
    }
    /// Get a collection of the checkbox controls.
    ///
    fn controls(&self) -> Vec<&Checkbox> {
        as_refs!(self.fields)
    }
    /// Draw the group of checkbox controls on the terminal screen.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the group will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` contains the [active and normal styles](ActiveNormalStyles) used to draw the checkbox.
    ///
    fn render(&self, mut area: Rect, buffer: &mut Buffer, styles: ActiveNormalStyles) -> Option<Position> {
        log_render!("CheckBoxGroup");
        // show the title if one has been set
        if let Some(title) = &self.title {
            let title_style = match self.active {
                true => styles.active.get(StyleId::GroupTitle),
                false => styles.normal.get(StyleId::GroupTitle),
            };
            let line = Line::from(title.as_str()).style(title_style);
            Block::new()
                .title(Title::from(line))
                .title_alignment(self.title_alignment)
                .borders(Borders::NONE)
                .render(area, buffer);
            area = inner_rect(area, (1, 1), (-1, 0));
        }
        if self.center_fields {
            let size = Size {
                width: self.fields.iter().map(|f| f.size().width).max().unwrap_or(0),
                height: self.fields.len() as u16,
            };
            area = center(area, size);
        }
        // only the checkboxes determine the visible height here
        let visible_height = cmp::min(area.height as usize, self.fields.len());
        area.height = 1;
        let mut coord = None;
        for idx in 0..visible_height {
            let checkbox = &self.fields[idx];
            let catalog = if checkbox.is_active() { styles.active } else { styles.normal };
            if let Some(field_coord) = checkbox.render(area, buffer, catalog) {
                coord.replace(field_coord);
            }
            area.y += 1;
        }
        coord
    }
    /// Set all checkbox controls in the group not active.
    ///
    fn clear_active(&mut self) {
        for field in self.fields.iter_mut() {
            field.set_active(false);
        }
    }
    /// Set the first checkbox control in the group active.
    ///
    fn set_first_active(&mut self) {
        self.fields.first_mut().unwrap().set_active(true);
        self.fields[1..].iter_mut().for_each(|field| field.set_active(false));
    }
    /// Set the last checkbox control in the group active.
    ///
    fn set_last_active(&mut self) {
        self.fields.last_mut().unwrap().set_active(true);
        let len = self.fields.len() - 1;
        self.fields[..len].iter_mut().for_each(|field| field.set_active(false));
    }
}
