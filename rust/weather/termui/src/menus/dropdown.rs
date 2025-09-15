//! The internal API used by menu implementors that use a [dropdown menu](DropdownMenu).
//!
use super::*;

impl DropdownMenu {
    /// Create a new dropdown menu from the collection of menu items.
    ///
    /// # Arguments
    ///
    /// - `menu_items` defines the contents of the dropdown menu.
    ///
    pub(in crate::menus) fn new(mut menu_items: Vec<MenuItem>) -> Self {
        debug_assert!(menu_items.len() > 0, "MenuItem collection cannot be empty");
        menu_items.iter_mut().for_each(|item| item.set_bordered());
        // account for the surrounding border
        let width = menu_items.iter().max_by(|lhs, rhs| lhs.width.cmp(&rhs.width)).map(|item| item.width).unwrap();
        let height = menu_items.len() as u16;
        Self { menu_items, size: Size { width: width + 2, height: height + 2 } }
    }
    /// Get the size of the dropdown menu.
    ///
    pub(in crate::menus) fn size(&self) -> Size {
        self.size
    }
    /// [Reset](Self::reset()) the menu items and set the first item to an [Active](MenuState) state.
    ///
    pub(in crate::menus) fn initialize(&mut self) {
        self.reset();
        self.menu_items.first_mut().unwrap().set_state(MenuState::Active);
    }
    /// Draw the menu items on the terminal screen surrounded by a box and return the active cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the dropdown menu will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` are used to draw the menu items.
    ///
    pub(in crate::menus) fn render(
        &self,
        area: Rect,
        buffer: &mut Buffer,
        styles: &ActiveNormalStyles,
    ) -> Option<Position> {
        log_render!("DropdownMenu");
        // scope the draw area to what the menu needs
        let draw_area = inner_rect(area, (0, 0), (self.size.width as i32, self.size.height as i32));
        if draw_area.is_empty() {
            None?;
        }
        // draw the menu border
        Clear::default().render(draw_area, buffer);
        if draw_area.height == 1 {
            Block::default().borders(Borders::TOP)
        } else if draw_area.height < self.size.height {
            Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        } else {
            Block::default().borders(Borders::ALL)
        }
        .border_style(match self.state() {
            MenuState::Active => styles.active.get(StyleId::DialogBorder),
            _ => styles.normal.get(StyleId::DialogBorder),
        })
        .render(draw_area, buffer);
        // draw the items that aren't active
        let menu_area = inner_rect(draw_area, (1, 1), (-1, -1));
        for y_offset in 0..self.menu_items.len() {
            let menu_item = &self.menu_items[y_offset];
            if menu_item.state() != MenuState::Passive {
                continue;
            }
            // scope the draw area to just the label
            let item_area = inner_rect(menu_area, (0, y_offset as i32), (0, (y_offset + 1) as i32));
            if item_area.is_empty() {
                break;
            }
            menu_item.render(item_area, buffer, styles);
        }
        // draw the active menu item
        let mut position = None;
        // for idx in 0..self.size.height as usize {
        for item_row in 0..self.menu_items.len() {
            let menu_item = &self.menu_items[item_row];
            if menu_item.state() != MenuState::Passive {
                // let active_area = inner_rect(area, (0, idx as i32), (0, 0));
                let active_area = inner_rect(area, (1, (1 + item_row) as i32), (0, 0));
                position = menu_item.render(active_area, buffer, styles);
                break;
            }
        }
        position
    }
    /// Dispatch a key pressed event to the collection of menu items. [ControlFlow::Continue] will be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    pub(in crate::menus) fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!("DropdownMenu");
        macro_rules! find {
            ($state:expr) => {
                self.menu_items.iter_mut().find(|menu_item| menu_item.state() == $state)
            };
        }
        // match self.find(MenuState::Selected) {
        match find!(MenuState::Selected) {
            // pass the event onto the next selected menu item
            Some(selected_item) => {
                selected_item.key_pressed(key_event)?;
            }
            // active and passive states work the same for movement
            None => match (key_event.modifiers, key_event.code) {
                (KeyModifiers::NONE, KeyCode::Down | KeyCode::Tab) => self.next(key_event.code == KeyCode::Tab)?,
                (KeyModifiers::NONE, KeyCode::Up) => self.previous(false)?,
                (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::BackTab) => self.previous(true)?,
                // _ => match self.find(MenuState::Active) {
                _ => match find!(MenuState::Active) {
                    Some(active_item) => {
                        let result = active_item.key_pressed(key_event);
                        result?;
                    }
                    None => {
                        for menu_item in self.menu_items.iter_mut() {
                            let result = menu_item.key_pressed(key_event);
                            if ControlFlow::Break(ControlResult::Cancel) == result {
                                menu_item.set_state(MenuState::Active);
                            }
                            result?;
                        }
                    }
                },
            },
        }
        ControlFlow::Continue(())
    }
    /// Set the active item to the next menu item in the collection.
    ///
    /// # Arguments
    ///
    /// - `wrap` will set the first menu item active if the last item is currently active.
    /// [NotAllowed](ControlResult::NotAllowed) will be returned in the active item cannot be moved otherwise
    /// [Continue](ControlResult::Continue) is returned.
    ///
    fn next(&mut self, wrap: bool) -> ControlFlow<ControlResult> {
        let menu_len = self.menu_items.len();
        #[cfg(debug_assertions)]
        log::debug!("DropdownMenu next, len={}", menu_len);
        if menu_len < 2 {
            break_event!(ControlResult::NotAllowed)?;
        }
        // get the index of the active menu item
        match self.active_index() {
            None => {
                #[cfg(debug_assertions)]
                log::error!("DropdownMenu is not active!\n{:#?}", self);
                break_event!(ControlResult::NotAllowed)
            }
            Some(mut active_index) => match active_index == (menu_len - 1) {
                // make sure you can move from last to first
                true => match wrap {
                    false => break_event!(ControlResult::NotAllowed),
                    true => {
                        self.menu_items.first_mut().unwrap().set_state(MenuState::Active);
                        self.menu_items.last_mut().unwrap().set_state(MenuState::Passive);
                        break_event!(ControlResult::Continue)
                    }
                },
                false => {
                    active_index += 1;
                    self.menu_items.iter_mut().enumerate().for_each(|(index, menu_item)| match index == active_index {
                        true => menu_item.set_state(MenuState::Active),
                        false => menu_item.set_state(MenuState::Passive),
                    });
                    break_event!(ControlResult::Continue)
                }
            },
        }
    }
    /// Set the active item to the previous menu item in the collection.
    ///
    /// # Arguments
    ///
    /// - `wrap` will set the last menu item active if the first item is currently active.
    /// [NotAllowed](ControlResult::NotAllowed) will be returned in the active item cannot be moved otherwise
    /// [Continue](ControlResult::Continue) is returned.
    ///
    fn previous(&mut self, wrap: bool) -> ControlFlow<ControlResult> {
        let menu_len = self.menu_items.len();
        #[cfg(debug_assertions)]
        log::debug!("DropdownMenu previous, len={}", menu_len);
        if menu_len < 2 {
            break_event!(ControlResult::NotAllowed)?;
        }
        // get the index of the active menu item
        match self.active_index() {
            None => {
                #[cfg(debug_assertions)]
                log::error!("DropdownMenu is not active!\n{:#?}", self);
                break_event!(ControlResult::NotAllowed)
            }
            Some(mut active_index) => match active_index == 0 {
                // make sure you can move from first to last
                true => match wrap {
                    false => break_event!(ControlResult::NotAllowed),
                    true => {
                        self.menu_items.first_mut().unwrap().set_state(MenuState::Passive);
                        self.menu_items.last_mut().unwrap().set_state(MenuState::Active);
                        break_event!(ControlResult::Continue)
                    }
                },
                false => {
                    active_index -= 1;
                    self.menu_items.iter_mut().enumerate().for_each(|(index, menu_item)| match index == active_index {
                        true => menu_item.set_state(MenuState::Active),
                        false => menu_item.set_state(MenuState::Passive),
                    });
                    break_event!(ControlResult::Continue)
                }
            },
        }
    }
    /// A helper function that returns the index of the currently [Active](MenuState::Active) menu item.
    ///
    #[inline]
    fn active_index(&self) -> Option<usize> {
        self.menu_items.iter().position(|menu_item| menu_item.state() == MenuState::Active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigate() {
        let mut testcase = DropdownMenu::new(vec![
            MenuItem::new("action", "Action", 'A'),
            MenuItem::new("submenu", "SubMenu", 'S')
                .with_menu(vec![MenuItem::new("action1", "Action1", '1').with_char_select()]),
        ]);
        macro_rules! key_event {
            ($code:expr, $modifiers:expr) => {
                &KeyEvent::new($code, $modifiers)
            };
            ($code:expr) => {
                &KeyEvent::new($code, KeyModifiers::NONE)
            };
        }
        // try moving without anything active
        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Tab)), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.key_pressed(key_event!(KeyCode::BackTab)), break_event!(ControlResult::NotAllowed));
        // select the action
        testcase.initialize();
        assert_eq!(
            testcase.key_pressed(key_event!(KeyCode::Enter)),
            break_event!(ControlResult::Selected("action".to_string()))
        );
        // select the submenu action
        testcase.reset();
        assert_eq!(
            testcase.key_pressed(key_event!(KeyCode::Char('S'), KeyModifiers::ALT)),
            break_event!(ControlResult::Continue)
        );
        assert_eq!(
            testcase.key_pressed(key_event!(KeyCode::Char('1'))),
            break_event!(ControlResult::Selected("action1".to_string()))
        );
        // select the submenu via cursor
        testcase.reset();
        testcase.initialize();
        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Down)), break_event!(ControlResult::Continue));
        assert_eq!(testcase.menu_items.last().unwrap().state(), MenuState::Active);
        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Right)), break_event!(ControlResult::Continue));
        assert_eq!(testcase.menu_items.last().unwrap().state(), MenuState::Selected);
        assert_eq!(testcase.menu_items.last().unwrap().menu.as_ref().unwrap().state(), MenuState::Active);
        assert_eq!(
            testcase.key_pressed(key_event!(KeyCode::Enter)),
            break_event!(ControlResult::Selected("action1".to_string()))
        );
        // verify wrapping
        testcase.reset();
        testcase.initialize();
        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Up)), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.key_pressed(key_event!(KeyCode::BackTab)), break_event!(ControlResult::Continue));
        assert_eq!(testcase.menu_items.last().unwrap().state(), MenuState::Active);
        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Down)), break_event!(ControlResult::NotAllowed));
        assert_eq!(testcase.key_pressed(key_event!(KeyCode::Tab)), break_event!(ControlResult::Continue));
    }
}
