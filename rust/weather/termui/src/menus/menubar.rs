//! The TUI menubar implementation.
//!
//! The [Menubar] manages a horizontal collection of [MenuItem] controls.
//!
use super::*;

const SEPARATOR_WIDTH: u16 = 1;

/// A horizontal, borderless menu.
///
#[derive(Debug)]
pub struct Menubar {
    /// The collection of menu items.
    menu: Vec<MenuItem>,
    /// The size of the menu.
    size: Size,
    /// The type of style catalog to use when drawing the menu.
    pub catalog_type: CatalogType,
}
impl Menubar {
    /// Create a new menubar.
    ///
    /// # Arguments
    ///
    /// - `menu` is the collection of menu items used by the menubar.
    ///
    pub fn new(mut menu: Vec<MenuItem>) -> Self {
        debug_assert!(menu.len() > 0, "Menubar menu cannot be empty.");
        // a menu action will always have a width
        let mut width = 0u16;
        for menu_item in menu.iter_mut() {
            if width > 0 {
                width += SEPARATOR_WIDTH;
            }
            width += menu_item.width();
            menu_item.reset();
        }
        Self { menu, size: Size { width, height: 1 }, catalog_type: CatalogType::MenuBar }
    }
    /// Indicates one of the menu item is not in a [Passive](MenuState::Passive) state.
    ///
    pub fn is_active(&self) -> bool {
        // you only need to look at the top level
        self.menu.iter().any(|menu_item| menu_item.state() != MenuState::Passive)
    }
    /// Used to [reset](MenuItem::reset()) each menu item.
    ///
    pub fn reset(&mut self) {
        self.menu.iter_mut().for_each(|menu_item| menu_item.reset());
    }
    /// Returns the size of the menubar.
    ///
    pub fn size(&self) -> Size {
        self.size
    }
    /// Draw the menubar on the terminal screen and return the active cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the menubar will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` are used to draw the menubar.
    ///
    pub fn render(&self, area: Rect, buffer: &mut Buffer, styles: ActiveNormalStyles) -> Option<Position> {
        log_render!("Menubar");
        // the area needs to be larger than the button bar to support cascading menus
        // so grab the first row for actions
        let mut item_area = inner_rect(area, (0, 0), (0, 1));
        // render all the menu item that are not active
        for item in &self.menu {
            if item_area.x > item_area.right() {
                break;
            } else if item.state() == MenuState::Passive {
                item.render(item_area, buffer, &styles);
            }
            item_area.x += item.width() + SEPARATOR_WIDTH;
        }
        // now you can render the active menu
        item_area = inner_rect(area, (0, 0), (0, 0));
        let mut active_position = None;
        for item in &self.menu {
            if item_area.right() > item_area.right() {
                break;
            } else if item.state() == MenuState::Passive {
                item_area.x += item.width() + SEPARATOR_WIDTH;
            } else {
                if let Some(position) = item.render(item_area, buffer, &styles) {
                    active_position.replace(position);
                }
                break;
            }
        }
        active_position
    }
    /// Dispatch a key pressed event to the collection of menu items. [ControlFlow::Continue] will be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    pub fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!("Menubar");
        let result = match self.menu.iter_mut().find(|item| item.state() != MenuState::Passive) {
            Some(active_item) => active_item.key_pressed(&key_event),
            None => {
                for item in self.menu.iter_mut() {
                    item.key_pressed(&key_event)?;
                }
                ControlFlow::Continue(())
            }
        };
        match result {
            // reset the menu when something has been selected
            ControlFlow::Break(ControlResult::Selected(_)) => {
                self.reset();
            }
            // don't pass on the menu item being canceled
            ControlFlow::Break(ControlResult::Cancel) => {
                self.reset();
                break_event!(ControlResult::Continue)?;
            }
            // if the key wasn't consumed check to see if you should move to another menu item
            ControlFlow::Continue(_) => match (key_event.modifiers, key_event.code) {
                (KeyModifiers::NONE, KeyCode::Tab) => match self.menu.len() < 2 {
                    true => break_event!(ControlResult::NotAllowed)?,
                    false => match self.menu.last().unwrap().state() != MenuState::Passive {
                        true => {
                            self.menu.last_mut().unwrap().set_state(MenuState::Passive);
                            self.menu.first_mut().unwrap().set_state(MenuState::Active);
                            break_event!(ControlResult::Continue)?;
                        }
                        false => match self.menu.iter().position(|item| item.state() != MenuState::Passive) {
                            None => break_event!(ControlResult::NotAllowed)?,
                            Some(active_index) => {
                                self.set_active(active_index + 1);
                                break_event!(ControlResult::Continue)?;
                            }
                        },
                    },
                },
                (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::BackTab) => match self.menu.len() < 2 {
                    true => break_event!(ControlResult::NotAllowed)?,
                    false => match self.menu.first().unwrap().state() != MenuState::Passive {
                        true => {
                            self.menu.last_mut().unwrap().set_state(MenuState::Active);
                            self.menu.first_mut().unwrap().set_state(MenuState::Passive);
                            break_event!(ControlResult::Continue)?;
                        }
                        false => match self.menu.iter_mut().position(|item| item.state() != MenuState::Passive) {
                            None => break_event!(ControlResult::NotAllowed)?,
                            Some(active_index) => {
                                self.set_active(active_index - 1);
                                break_event!(ControlResult::Continue)?;
                            }
                        },
                    },
                },
                _ => (),
            },
            _ => (),
        }
        result
    }
    /// An internal helper that sets the menu item, at the provided index, [Active](MenuState::Active). Other
    /// menu items will be set [Passive](MenuState::Passive).
    #[inline]
    fn set_active(&mut self, active_index: usize) {
        self.menu.iter_mut().enumerate().for_each(|(index, item)| {
            item.set_state(match index == active_index {
                true => MenuState::Active,
                false => MenuState::Passive,
            })
        });
    }
}
