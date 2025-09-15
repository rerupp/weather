//! The TUI implementation of a popup menu.
//!
//! The [PopupMenu] provides a vertical collection of [MenuItem] controls. It's
//! used to provide a contextual menu on demand.
//!
use controls::ControlResult;
use styles::CatalogType;

use super::*;

/// A bordered menu used to support context menus.
#[derive(Debug)]
pub struct PopupMenu {
    /// Used to track if the menu is currently active or not.
    active: bool,
    /// The context menu.
    menu: DropdownMenu,
    /// The type of style catalog to use when drawing the menu.
    catalog_type: CatalogType,
}
impl PopupMenu {
    /// Create a new popup menu.
    ///
    /// # Arguments
    ///
    /// - `items` is the collection of menu items used by the popup menu.
    ///
    pub fn new(items: Vec<MenuItem>) -> Self {
        let mut menu = DropdownMenu::new(items);
        menu.initialize();
        Self { active: false, menu, catalog_type: CatalogType::PopupMenu }
    }
    /// A builder method that set the popup menu into an active state.
    ///
    pub fn with_active(mut self) -> Self {
        self.active = true;
        self
    }
    /// Resets the into an pristine state.
    ///
    pub fn reset(&mut self) {
        self.active = false;
        self.menu.initialize();
    }
    /// Gets the size of the popup menu.
    ///
    pub fn size(&self) -> Size {
        self.menu.size()
    }
    /// Queries if the popup menu is active or not.
    ///
    pub fn is_active(&self) -> bool {
        self.active
    }
    /// Set the state of the popup menu.
    ///
    /// # Arguments
    ///
    /// - `yes_no` controls if the popup menu is active or not.
    ///
    pub fn set_active(&mut self, yes_no: bool) {
        self.active = yes_no;
    }
    /// Dispatch a key pressed event to the collection of menu items. [ControlFlow::Continue] will be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    pub fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!("PopupMenu");
        self.menu.key_pressed(&key_event)
    }
    /// Draw the popup menu on the terminal screen and return the active cursor position.
    ///
    /// # Arguments
    ///
    /// - `position` is the upper left coordinate where the popup menu should be drawn.
    /// - `area` is where on the terminal screen the menubar can be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    pub fn render(&self, position: Position, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        if !area.contains(position) {
            None?;
        }
        log_render!("PopupMenu");
        // does the popup need to be clipped
        let size = self.menu.size();
        let popup_area = if size.height > area.height {
            let upper_bounds = (position.x as i32, 0);
            let lower_bounds = ((position.x + size.width) as i32, 0);
            inner_rect(area, upper_bounds, lower_bounds)
        // will the popup fit within the area
        } else if (position.y + size.height) < area.bottom() {
            let y_offset = position.y.saturating_sub(area.y);
            let upper_bounds = (position.x as i32, y_offset as i32);
            let lower_bounds = ((position.x + size.width) as i32, (y_offset + size.height) as i32);
            inner_rect(area, upper_bounds, lower_bounds)
        // otherwise it will fit but needs to be rendered bottom up
        } else {
            let upper_bounds = (position.x as i32, -(size.height as i32));
            let lower_bounds = ((position.x + size.width) as i32, 0);
            inner_rect(area, upper_bounds, lower_bounds)
        };
        self.menu.render(popup_area, buffer, &ActiveNormalStyles::new(self.catalog_type))
    }
}
