//! The UI menu implementations.
//!
//! There are 2 types of menus available.
//!
//! - A [Menubar] manages a horizontal collection of [MenuItem] controls.
//! - A [PopupMenu] manages a vertical collection of [MenuItem] controls.
//!
//! This is the first time I've' split the public `API` and internal `API` into
//! separate modules. I'm still trying to decide if I like this pattern. Regardless
//! supporting a public and internal `API` requires a lot of *`pub(crate)`* source
//! code markup where a concept like *`internal`* would be nice.
//!
use super::*;
use controls::ControlResult;
use styles::{ActiveNormalStyles, CatalogType, StyleId};

pub use menubar::Menubar;
mod menubar;

pub use popup::PopupMenu;
mod popup;

mod dropdown;
mod menuitem;

/// The state of a [menu item](MenuItem).
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuState {
    /// The menu item is not considered active or selected.
    Passive,
    /// The menu item is currently active.
    Active,
    /// The menu item is currently selected.
    Selected,
}
impl std::fmt::Display for MenuState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// The menu item is a basic building block of other menu types.
///
/// Each menu item has an identifier, description, and selector key. It can include a
/// submenu allowing it to support cascading menus.
///
/// The menu item includes a public builder `API` to create instances and an internal `API`
/// used by other menu implementations.
///
#[derive(Debug)]
pub struct MenuItem {
    /// Track the state of the menu item.
    state: MenuState,
    /// The menu item identifier.
    id: String,
    /// The menu item description.
    label: String,
    /// The width of the label.
    width: u16,
    /// The character that can select the item.
    selector: char,
    /// The lowercase value of the selector character.
    selector_lc: String,
    /// The x offset of the labels selector character.
    selector_offset: u16,
    /// Allow the menu item to be selected by without using `ALT`.
    char_select: bool,
    /// The optional submenu.
    menu: Option<DropdownMenu>,
    /// Tracks if there is a border surrounding the label.
    bordered: bool,
}
impl std::fmt::Display for MenuItem {
    /// Let the menu item to display the list of menu items.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MenuItem[{}] {} {}",
            self.id,
            match self.menu.is_some() {
                true => "submenu",
                false => "action",
            },
            self.state,
        )
    }
}
/// The builder view of a menu item.
impl MenuItem {
    /// Creates a menu item.
    ///
    /// # Arguments
    ///
    /// - `id` is the menu item identification.
    /// - `label` is the menu item description.
    /// - `selector` is the character which selects the menu item.
    ///
    pub fn new(id: impl ToString, label: impl ToString, selector: char) -> Self {
        let label = label.to_string();
        let id = id.to_string();
        let label = label.to_string();
        let width = label.len() as u16;
        debug_assert!(id.len() > 0, "MenuItem id is empty.");
        debug_assert!(width > 0, "MenuItem label is empty.");
        debug_assert!(selector != '\0', "MenuItem selector is 0.");
        let selector_offset = match label.chars().position(|ch| ch == selector) {
            None => {
                log::debug!("Selector '{}' not found in label '{}'!!!", selector, label);
                0
            }
            Some(offset) => offset as u16,
        };
        Self {
            state: MenuState::Passive,
            id,
            label,
            // There is a 1 character left,right label margin
            width: width + 2,
            selector,
            selector_lc: selector.to_lowercase().to_string(),
            selector_offset,
            char_select: false,
            menu: None,
            bordered: false,
        }
    }
    /// Adds a submenu to the menu item.
    ///
    /// # Arguments
    ///
    /// - `menu` is the submenu that will be added.
    ///
    pub fn with_menu(mut self, menu: Vec<MenuItem>) -> Self {
        self.menu.replace(DropdownMenu::new(menu));
        self
    }
    /// Configure the menu item to be selected by pressing the selector character without pressing the `ALT` key.
    ///
    pub fn with_char_select(mut self) -> Self {
        self.char_select = true;
        self
    }
    /// Indicates the menu item has a border around it.
    ///
    fn set_bordered(&mut self) {
        self.bordered = true;
        if let Some(menu) = self.menu.as_mut() {
            menu.menu_items.iter_mut().for_each(|item| item.set_bordered());
        };
    }
}

#[cfg(test)]
mod menuitem_tests {
    use super::*;

    #[test]
    fn menuitem() {
        let id = "id";
        let label = "laBel";
        let selector = 'B';
        let mut testcase = MenuItem::new(id.to_string(), label.to_string(), selector);
        assert_eq!(testcase.state, MenuState::Passive);
        assert_eq!(testcase.id, id);
        assert_eq!(testcase.label, label);
        assert_eq!(testcase.width as usize, label.len() + 2);
        assert_eq!(testcase.selector, selector);
        assert_eq!(testcase.selector_lc, "b");
        assert_eq!(testcase.selector_offset, 2);
        assert_eq!(testcase.char_select, false);
        testcase = testcase.with_char_select();
        assert_eq!(testcase.char_select, true);
        assert!(testcase.menu.is_none());
        testcase = testcase.with_menu(vec![MenuItem::new("id", "item", 'i')]);
        assert!(testcase.menu.is_some());
    }
}

/// The dropdown menu manages a collection of [menu items](MenuItem).
///
#[derive(Debug)]
struct DropdownMenu {
    /// The menu items that comprise the dropdown menu.
    menu_items: Vec<MenuItem>,
    /// The size of the dropdown menu.
    size: Size,
}
impl std::fmt::Display for DropdownMenu {
    /// Show the dropdown menu as an array of menu items.
    ///
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DropdownMenu [\n")?;
        for item in self.menu_items.iter() {
            write!(f, "  {},\n", item)?;
        }
        write!(f, "]")
    }
}
impl DropdownMenu {
    /// Return the [state](MenuState) of the menu.
    ///
    fn state(&self) -> MenuState {
        self.menu_items
            .iter()
            .find(|item| item.state() != MenuState::Passive)
            .map_or(MenuState::Passive, |item| item.state())
    }
    /// Each of the menu items will be [reset](MenuItem::reset()).
    ///
    fn reset(&mut self) {
        self.menu_items.iter_mut().for_each(|item| item.reset());
    }
}

#[cfg(test)]
mod dropdown_tests {
    use super::*;

    #[test]
    fn initialize() {
        let mut testcase = DropdownMenu::new(vec![
            MenuItem::new("action", "Action", 'A'),
            MenuItem::new("submenu", "SubMenu", 'S').with_menu(vec![
                MenuItem::new("action1", "Action1", '1'),
                MenuItem::new("action2", "Action2", '2'),
                MenuItem::new("action3", "Action3", '3'),
            ]),
        ]);
        // remember there is left/right whitespace + a border
        assert_eq!(testcase.size, Size { width: ("SubMenu".len() + 4) as u16, height: 4 });
        assert_eq!(
            testcase.menu_items.last().unwrap().menu.as_ref().unwrap().size,
            Size { width: ("Action#".len() + 4) as u16, height: 5 }
        );
        assert_eq!(testcase.state(), MenuState::Passive);
        testcase.initialize();
        assert_eq!(testcase.state(), MenuState::Active);
        assert_eq!(testcase.menu_items.first().unwrap().state(), MenuState::Active);
        testcase.reset();
        assert_eq!(testcase.state(), MenuState::Passive);
        assert_eq!(testcase.menu_items.first().unwrap().state(), MenuState::Passive);
    }
}
