//! The TUI tab dialog.
//!
//! The [TabDialog] allows multiple [tab windows](TabWindow) to be shown, one at a
//! time, on the screen, The dialog is divided into a *tab* area at the top of the
//! screen with the window area underneath occupying the rest of the screen.
//! The active `TabWindow` is drawn within the available window area. Each
//!
//! The `TabDialog` allows windows to be dynamically added and removed.
//!
use super::*;
use controls::hotkey_spans;
use ratatui::{layout::Position, symbols::border::Set};
use styles::{ActiveNormalStyles, CatalogType};

/// A window managed by the tab dialog.
///
#[derive(Debug)]
pub struct TabWindow {
    /// The tab window identifier.
    id: String,
    /// The description of the tab window.
    title: String,
    /// The character that selects the tab window.
    selector: char,
    /// The selector character forced to lowercase.
    lc_selector: String,
    /// The managed dialog window.
    win: Box<dyn DialogWindow>,
}
impl TabWindow {
    /// Create a new instance of the tab dialog window.
    ///
    /// # Arguments
    ///
    /// - `id` is the unique tab identifier.
    /// - `title` is the tab window description.
    /// - `selector` is the character that will select the tab window.
    /// - `win` is the managed dialog window.
    ///
    pub fn new(id: impl ToString, title: impl ToString, selector: char, win: impl DialogWindow + 'static) -> Self {
        let title = format!(" {} ", title.to_string().trim());
        let lc_selector = selector.to_lowercase().to_string();
        Self { title, selector, lc_selector, id: id.to_string(), win: Box::new(win) }
    }
}

/// The collection of tab dialog windows.
#[derive(Debug, Default)]
struct Windows(Vec<TabWindow>);
impl Windows {
    /// If the tab window exists in the collection replace it otherwise add it to the collection.
    ///
    /// # Arguments
    ///
    /// - `tab_window` is the window that will be updated or added to the collection.
    ///
    fn add_or_replace(&mut self, tab_window: TabWindow) {
        match self.position(&tab_window.id) {
            None => self.0.push(tab_window),
            Some(index) => self.0[index] = tab_window,
        }
    }
    /// Get the index of a tab window in the collection.
    ///
    /// # Arguments
    ///
    /// - `id` is the tab window identifier.
    ///
    fn position(&self, id: &str) -> Option<usize> {
        self.0.iter().position(|tab| tab.id.as_str() == id)
    }
    /// Get a reference to the currently active tab window.
    ///
    fn active(&self) -> Option<&TabWindow> {
        self.0.iter().find(|tab| tab.win.is_active())
    }
    /// Get a mutable reference to the currently active tab window.
    ///
    fn active_mut(&mut self) -> Option<&mut TabWindow> {
        self.0.iter_mut().find(|tab| tab.win.is_active())
    }
    /// Get the index of the active tab window in the collection.
    ///
    fn active_index(&self) -> Option<usize> {
        self.0.iter().position(|tab| tab.win.is_active())
    }
    /// Set the active tab.
    ///
    /// # Arguments
    ///
    /// - `id` is the tab identifier.
    ///
    fn set_active(&mut self, id: impl ToString) {
        let id = id.to_string();
        if let Some(position) = self.position(&id) {
            self.0.iter_mut().enumerate().for_each(|(index, tab)| tab.win.set_active(index == position))
        }
    }
    /// Remove the currently active tab from the dialog.
    ///
    fn remove_active(&mut self) {
        if let Some(mut active_index) = self.active_index() {
            self.0.remove(active_index);
            active_index = cmp::min(active_index, self.0.len().saturating_sub(1));
            self.0.iter_mut().enumerate().for_each(|(index, tab)| tab.win.set_active(index == active_index));
        }
    }
    /// Move the active tab window to the next window in the collection.
    ///
    /// # Arguments
    ///
    /// - `wrap` indicates the first tab should be set active if the last tab is currently active.
    ///
    fn next_tab(&mut self, wrap: bool) -> bool {
        // make sure there are at least 2 tabs
        match self.0.len() < 2 {
            true => false,
            // special case the last tab being active
            false => match self.0.last().unwrap().win.is_active() {
                true => match wrap {
                    false => false,
                    true => {
                        self.0.last_mut().unwrap().win.set_active(false);
                        self.0.first_mut().unwrap().win.set_active(true);
                        true
                    }
                },
                false => match self.active_index() {
                    None => false,
                    Some(index) => {
                        // there will always be at least 2 tabs when you get here
                        self.0.get_mut(index).unwrap().win.set_active(false);
                        self.0.get_mut(index + 1).unwrap().win.set_active(true);
                        true
                    }
                },
            },
        }
    }
    /// Move the active tab window to the previous window in the collection.
    ///
    /// # Arguments
    ///
    /// - `wrap` indicates the last tab should be set active if the first tab is currently active.
    ///
    fn prev_tab(&mut self, wrap: bool) -> bool {
        // make sure there are at least 2 tabs
        match self.0.len() < 2 {
            true => false,
            // special case the first tab being active
            false => match self.0.first().unwrap().win.is_active() {
                true => match wrap {
                    false => false,
                    true => {
                        self.0.first_mut().unwrap().win.set_active(false);
                        self.0.last_mut().unwrap().win.set_active(true);
                        true
                    }
                },
                false => match self.active_index() {
                    None => false,
                    Some(index) => {
                        // there will always be at least 2 tabs when you get here
                        self.0.get_mut(index).unwrap().win.set_active(false);
                        self.0.get_mut(index - 1).unwrap().win.set_active(true);
                        true
                    }
                },
            },
        }
    }
    /// Set the active tab if selected by an `ALT` key event.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("Windows");
        match (key_event.modifiers, key_event.code) {
            (KeyModifiers::ALT, KeyCode::Char(selector)) => {
                let selector = selector.to_lowercase().to_string();
                if let Some(position) = self.0.iter().position(|tab| selector == tab.lc_selector) {
                    self.0.iter_mut().enumerate().for_each(|(index, tab)| tab.win.set_active(index == position));
                    break_event!(DialogResult::Continue)?;
                }
            }
            _ => (),
        }
        ControlFlow::Continue(())
    }
}

/// A multi-window dialog with each window shown within a selectable tab.
#[derive(Debug)]
pub struct TabDialog {
    /// Track if the tab dialog is in an active state.
    active: bool,
    /// The windows managed by the dialog.
    windows: Windows,
    /// Any window errors that are encountered.
    message: Option<MessageDialog>,
    /// The tab dialog style catalog type. This will always be [CatalogType::TabDialog].
    pub catalog_type: CatalogType,
}
impl TabDialog {
    /// Create a new instance of the dialog.
    ///
    pub fn new() -> Self {
        Self { active: false, windows: Windows::default(), message: None, catalog_type: CatalogType::TabDialog }
    }
    /// A builder method that adds a window.
    ///
    /// # Arguments
    ///
    /// - `tab` is the window that will be added to the dialog.
    ///
    pub fn with_tab(mut self, tab: TabWindow) -> Self {
        self.add_or_replace_tab(tab);
        self
    }
    /// Query if the dialog does not contain any windows.
    ///
    pub fn is_empty(&self) -> bool {
        self.windows.0.is_empty()
    }
    /// Query if the dialog contains a window tab.
    ///
    /// # Arguments
    ///
    /// - `id` is the tab window identifier.
    ///
    pub fn contains_tab(&self, id: impl ToString) -> bool {
        self.windows.position(&id.to_string()).is_some()
    }
    /// Set a window tab active.
    ///
    /// # Arguments
    ///
    /// - `id` is the tab window identifier.
    ///
    pub fn set_active_tab(&mut self, id: impl ToString) {
        self.windows.set_active(id);
    }
    /// Add or replace a window tab.
    ///
    /// # Arguments
    ///
    /// - `tab` is the window tab that will be added or replaced in the dialog.
    ///
    pub fn add_or_replace_tab(&mut self, tab: TabWindow) {
        let tab_id = tab.id.clone();
        self.windows.add_or_replace(tab);
        self.windows.set_active(tab_id);
    }
    /// Remove the active tab from the dialog.
    ///
    pub fn remove_active_tab(&mut self) {
        self.windows.remove_active();
    }
}
impl DialogWindow for TabDialog {
    /// Query if the tab dialog is active or not.
    ///
    fn is_active(&self) -> bool {
        self.active
    }
    /// Controls setting the tab dialog active or not.
    ///
    /// # Arguments
    ///
    /// - `yes_no` determines if the tab dialog is active or not.
    ///
    fn set_active(&mut self, yes_no: bool) {
        self.active = yes_no;
    }
    /// Refresh all the tab windows.
    ///
    fn refresh(&mut self) -> std::result::Result<(), String> {
        for tab in self.windows.0.iter_mut() {
            if let Err(error) = tab.win.refresh() {
                let error_message = format!("{} refresh error\n{}", tab.title, error);
                self.message.replace(MessageDialog::new(MessageStyle::Error, error_message));
                break;
            }
        }
        Ok(())
    }
    /// Get the size of the window.
    ///
    fn size(&self) -> Size {
        // the size of the dialog isn't known until render time
        Size::default()
    }
    /// Dispatch a key pressed event to the message dialog, active tab window, or tab dialog frame.
    /// [ControlFlow::Continue] should be returned if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        log_key_pressed!("TabDialog");
        match self.message.take() {
            Some(mut message) => {
                if ControlFlow::Break(DialogResult::Continue) == message.key_pressed(key_event) {
                    self.message.replace(message);
                }
                break_event!(DialogResult::Continue)?;
            }
            None => {
                // the active window needs first shot at consuming the event
                if let Some(tab) = self.windows.active_mut() {
                    tab.win.key_pressed(key_event)?;
                }
                macro_rules! move_tab {
                    ($move_tab:expr) => {{
                        if !$move_tab {
                            beep();
                        }
                        break_event!(DialogResult::Continue)?
                    }};
                }
                match (key_event.modifiers, key_event.code) {
                    (KeyModifiers::ALT, KeyCode::Left) => move_tab!(self.windows.prev_tab(false)),
                    (KeyModifiers::ALT, KeyCode::BackTab) => move_tab!(self.windows.prev_tab(true)),
                    (KeyModifiers::ALT, KeyCode::Right) => move_tab!(self.windows.next_tab(false)),
                    (KeyModifiers::ALT, KeyCode::Tab) => move_tab!(self.windows.next_tab(true)),
                    _ => self.windows.key_pressed(key_event)?,
                }
            }
        }
        ControlFlow::Continue(())
    }
    /// Draw the tab dialog on the terminal screen and optionally return the current cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the tab dialog will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        log_render!("TabDialog");
        // let styles = ActiveNormalStyles::new(self.catalog_type);
        let styles = match self.message.is_some() {
            // ActiveNormalStyles::new(self.catalog_type)
            true => ActiveNormalStyles::with_active_style(self.catalog_type, ControlState::Normal),
            false => ActiveNormalStyles::new(self.catalog_type),
        };
        // this needs to be done JIC the dialog isn't sitting in another dialog
        Clear::default().render(area, buffer);
        // set up the windows area
        let windows_area = inner_rect(area, (0, 2), (0, 0));
        Block::default()
            .borders(Borders::ALL)
            .border_style(styles.active.get(StyleId::DialogBorder))
            .style(styles.active.get(StyleId::Screen))
            .render(windows_area, buffer);
        let mut position = None;
        if let Some(tab) = self.windows.active() {
            let window_area = inner_rect(windows_area, (1, 1), (-1, -1));
            if let Some(window_position) = tab.win.render(window_area, buffer) {
                position.replace(window_position);
            }
        }
        // make sure the tab area gets styled
        let mut tab_area = inner_rect(area, (0, 0), (0, 2));
        Block::default().style(styles.active.get(StyleId::Screen)).render(tab_area, buffer);
        if self.windows.0.len() > 0 {
            // the tab area needs to overlay the top of the window area to redraw the top border
            tab_area.height += 1;
            // create the tab titles
            let text_style = styles.active.get(StyleId::LabelText);
            let selector_style = styles.active.get(StyleId::LabelSelector);
            // todo: next step create your own tabs
            let titles = self
                .windows
                .0
                .iter()
                .map(|tab| Line::from(hotkey_spans(&tab.title, tab.selector, text_style, selector_style)))
                .collect::<Vec<Line>>();
            let mut width = 2 + titles.iter().map(|line| line.width()).sum::<usize>();
            width += titles.len() - 1;
            tab_area.width = width as u16;
            let border_set = Set {
                bottom_left: symbols::line::VERTICAL_RIGHT,
                bottom_right: symbols::line::HORIZONTAL_UP,
                ..Default::default()
            };
            Tabs::new(titles)
                .block(
                    Block::new()
                        .borders(Borders::ALL)
                        .border_set(border_set)
                        .style(styles.active.get(StyleId::DialogBorder)),
                )
                .padding("", "")
                .highlight_style(styles.active.get(StyleId::Highlight))
                .style(styles.active.get(StyleId::Text))
                .select(self.windows.active_index().unwrap_or(0))
                .render(tab_area, buffer);
        }
        if let Some(message) = &self.message {
            if let Some(message_position) = message.render(area, buffer) {
                position.replace(message_position);
            }
        }
        position
    }
}
