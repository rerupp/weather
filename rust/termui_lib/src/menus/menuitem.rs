//! The internal `API` used by menu implementors that consume a [menu item](MenuItem).
//!
use super::*;

impl MenuItem {
    /// Get the width of the menu item.
    ///
    pub(in crate::menus) fn width(&self) -> u16 {
        self.width
    }
    /// Get the state of the menu item.
    ///
    pub(in crate::menus) fn state(&self) -> MenuState {
        self.state
    }
    /// Set the menu item and submenu to a [Passive](MenuState) state.
    ///
    pub(in crate::menus) fn reset(&mut self) {
        self.state = MenuState::Passive;
        if let Some(menu) = self.menu.as_mut() {
            menu.reset();
        }
    }
    /// Set the state of a menu item and submenu.
    ///
    /// # Arguments
    ///
    /// - `Passive` will [reset][Self::reset()] the menu item.
    /// - `Active` will set the menu item state [Active](MenuState) and [reset](DropdownMenu::reset()) the submenu.
    /// - `Selected` will set the menu item state [Selected](MenuState) and [initialize](DropdownMenu::initialize())
    /// the submenu.
    ///
    pub(in crate::menus) fn set_state(&mut self, state: MenuState) {
        match state {
            MenuState::Passive => self.reset(),
            MenuState::Active => {
                self.state = state;
                if let Some(menu) = self.menu.as_mut() {
                    menu.reset();
                }
            }
            MenuState::Selected => {
                self.state = state;
                if let Some(menu) = self.menu.as_mut() {
                    menu.initialize();
                }
            }
        }
    }
    /// Draw the menu item, and submenu if not in a [Passive](MenuState) state, on the terminal screen and
    /// return the active cursor position.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal screen the menu item will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` are used to draw the menu item.
    ///
    pub(in crate::menus) fn render(
        &self,
        area: Rect,
        buffer: &mut Buffer,
        styles: &ActiveNormalStyles,
    ) -> Option<Position> {
        log_render!(self.to_string());
        let (label_style, selector_style) = match self.state {
            MenuState::Passive => (styles.normal.get(StyleId::LabelText), styles.normal.get(StyleId::LabelSelector)),
            _ => (styles.active.get(StyleId::LabelText), styles.active.get(StyleId::LabelSelector)),
        };
        let margin_style = styles.normal.get(StyleId::LabelText);
        let mut label = vec![Span::from(" ").style(margin_style)];
        label.append(&mut controls::hotkey_spans(&self.label, self.selector, label_style, selector_style));
        label.push(Span::from(" ").style(margin_style));
        Paragraph::new(Line::from(label)).render(area, buffer);
        match self.state {
            MenuState::Passive => None,
            MenuState::Active => {
                if let Some(menu) = &self.menu {
                    let upper_x = (self.width + self.bordered as u16) as i32;
                    let menu_area = inner_rect(area, (upper_x, 0), (0, 0));
                    menu.render(menu_area, buffer, styles);
                }
                // there's a 1 character left margin so include that in the offset
                let position = Position { x: area.x + 1 + self.selector_offset, y: area.y };
                Some(position)
            }
            MenuState::Selected => {
                debug_assert!(self.menu.is_some(), "MenuItem selected and menu None!\n{:#?}", self);
                // there's a 1 character right margin so include that in the offset
                let upper_x = (self.width + self.bordered as u16) as i32;
                let menu_area = inner_rect(area, (upper_x, 0), (0, 0));
                self.menu.as_ref().unwrap().render(menu_area, buffer, styles)
            }
        }
    }
    /// Dispatch a key pressed event to the menu item. [ControlFlow::Continue] will be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    pub(in crate::menus) fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!(self.to_string());
        match self.state {
            MenuState::Passive => self.passive_key_pressed(key_event)?,
            MenuState::Active => self.active_key_pressed(key_event)?,
            MenuState::Selected => self.selected_key_pressed(key_event)?,
        }
        ControlFlow::Continue(())
    }
    /// Examine a key pressed event when the menu item is in a [Passive](MenuState) state.
    /// [ControlFlow::Continue] will be returned if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    fn passive_key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!("MenuItem", "passive_key_pressed");
        if self.is_selector(key_event) {
            self.state = MenuState::Selected;
            match self.menu.as_mut() {
                // if there is no menu then it is an action
                None => break_event!(ControlResult::Selected(self.id.to_string()))?,
                // the menu needs to get the event next time
                Some(menu) => {
                    menu.initialize();
                    break_event!(ControlResult::Continue)?;
                }
            }
        }
        ControlFlow::Continue(())
    }
    /// Examine a key pressed event when the menu item is in a [Active](MenuState) state.
    /// [ControlFlow::Continue] will be returned if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    fn active_key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
        log_key_pressed!("MenuItem", "active_key_pressed");
        match (key_event.modifiers, key_event.code) {
            (KeyModifiers::NONE, KeyCode::Right) => match self.menu.as_mut() {
                None => break_event!(ControlResult::NotAllowed)?,
                Some(menu) => {
                    self.state = MenuState::Selected;
                    menu.initialize();
                    break_event!(ControlResult::Continue)?;
                }
            },
            // allow the menu to dismiss itself
            (KeyModifiers::NONE, KeyCode::Left | KeyCode::Esc) => break_event!(ControlResult::Cancel)?,
            _ => {
                if self.is_selector(key_event) {
                    self.state = MenuState::Selected;
                    match self.menu.as_mut() {
                        None => break_event!(ControlResult::Selected(self.id.to_string()))?,
                        Some(menu) => {
                            menu.initialize();
                            break_event!(ControlResult::Continue)?;
                        }
                    }
                }
            }
        }
        ControlFlow::Continue(())
    }
    /// Examine a key pressed event when the menu item is in a [Selected](MenuState) state.
    /// [ControlFlow::Continue] will be returned if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    fn selected_key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
        // only items with a menu should be visited when selected
        debug_assert!(self.menu.is_some(), "MenuItem selected and menu None!\n{:#?}", self);
        log_key_pressed!("MenuItem", "selected_key_pressed");
        let menu = self.menu.as_mut().unwrap();
        match menu.key_pressed(key_event) {
            ControlFlow::Break(ControlResult::Cancel) => {
                self.state = MenuState::Active;
                if let Some(menu) = self.menu.as_mut() {
                    menu.reset();
                }
                break_event!(ControlResult::Continue)
            }
            ControlFlow::Continue(_) => {
                // if the menu didn't consume the event see if another menu item is being selected
                let mut selected_id = None;
                let mut result = ControlFlow::Continue(());
                for item in menu.menu_items.iter_mut() {
                    if item.is_selector(key_event) {
                        selected_id.replace(item.id.to_string());
                        item.set_state(MenuState::Selected);
                        result = match item.menu.is_some() {
                            true => break_event!(ControlResult::Continue),
                            false => break_event!(ControlResult::Selected(item.id.to_string())),
                        };
                        break;
                    }
                }
                if let Some(id) = selected_id {
                    menu.menu_items.iter_mut().for_each(|item| {
                        if item.id != id {
                            item.reset();
                        }
                    });
                }
                result
            }
            result => result,
        }
    }
    /// An internal helper that checks if the key event should select the menu item.
    ///
    /// # Arguments
    ///
    /// - `key_event` is the event to examine.
    ///
    fn is_selector(&self, key_event: &KeyEvent) -> bool {
        let mut is_selector = false;
        match key_event.code {
            KeyCode::Char(ch) => {
                if KeyModifiers::NONE == key_event.modifiers && self.char_select {
                    is_selector = ch.to_lowercase().to_string() == self.selector_lc;
                } else if KeyModifiers::ALT == key_event.modifiers {
                    is_selector = ch.to_lowercase().to_string() == self.selector_lc;
                }
            }
            _ => is_selector = KeyModifiers::NONE == key_event.modifiers && KeyCode::Enter == key_event.code,
        }
        is_selector
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states() {
        let mut testcase = MenuItem::new("id", "label", 'l')
            .with_menu(vec![MenuItem::new("mid1", "menu_item1", '1'), MenuItem::new("mid2", "menu_item2", '2')]);
        testcase.set_state(MenuState::Active);
        assert_eq!(testcase.state, MenuState::Active);
        assert_eq!(testcase.menu.as_ref().unwrap().state(), MenuState::Passive);
        testcase.set_state(MenuState::Selected);
        assert_eq!(testcase.state, MenuState::Selected);
        assert_eq!(testcase.menu.as_ref().unwrap().state(), MenuState::Active);
        testcase.reset();
        assert_eq!(testcase.state, MenuState::Passive);
        assert_eq!(testcase.menu.as_ref().unwrap().state(), MenuState::Passive);
        // make set is behaving as expected
        testcase.set_state(MenuState::Selected);
        testcase.set_state(MenuState::Active);
        assert_eq!(testcase.state, MenuState::Active);
        assert_eq!(testcase.menu.as_ref().unwrap().state(), MenuState::Passive);
        testcase.set_state(MenuState::Passive);
        assert_eq!(testcase.state, MenuState::Passive);
        assert_eq!(testcase.menu.as_ref().unwrap().state(), MenuState::Passive);
        testcase.set_state(MenuState::Selected);
        testcase.reset();
        assert_eq!(testcase.state, MenuState::Passive);
        assert_eq!(testcase.menu.as_ref().unwrap().state(), MenuState::Passive);
    }

    #[test]
    fn key_pressed() {
        // make sure the key event is passed onto the submenu
        let mut testcase = MenuItem::new("top", "Top", 'T').with_menu(vec![
            MenuItem::new("action", "Action", 'A'),
            MenuItem::new("menu", "Menu", 'M').with_menu(vec![MenuItem::new("sa", "SubAction", 'S')]),
        ]);
        let alt_a = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::ALT);
        let alt_t = KeyEvent::new(KeyCode::Char('T'), KeyModifiers::ALT);
        assert_eq!(testcase.key_pressed(&alt_a), ControlFlow::Continue(()));
        assert_eq!(testcase.key_pressed(&alt_t), break_event!(ControlResult::Continue));
        assert_eq!(testcase.key_pressed(&alt_t), ControlFlow::Continue(()));
        assert_eq!(testcase.key_pressed(&alt_a), break_event!(ControlResult::Selected("action".to_string())));
        // check selections
        testcase.set_state(MenuState::Passive);
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(testcase.key_pressed(&enter), break_event!(ControlResult::Continue));
        assert_eq!(testcase.state, MenuState::Selected);
        assert_eq!(testcase.menu.as_ref().unwrap().state(), MenuState::Active);
        assert_eq!(testcase.key_pressed(&enter), break_event!(ControlResult::Selected("action".to_string())));
        testcase.set_state(MenuState::Active);
        let right = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert_eq!(testcase.key_pressed(&right), break_event!(ControlResult::Continue));
        assert_eq!(testcase.state, MenuState::Selected);
        assert_eq!(testcase.menu.as_ref().unwrap().state(), MenuState::Active);
        let left = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(testcase.key_pressed(&left), break_event!(ControlResult::Continue));
        assert_eq!(testcase.state, MenuState::Active);
        assert_eq!(testcase.menu.as_ref().unwrap().state(), MenuState::Passive);
        // select the menu
        testcase.reset();
        testcase.set_state(MenuState::Selected);
        let key_event = KeyEvent::new(KeyCode::Char('M'), KeyModifiers::ALT);
        let result = testcase.key_pressed(&key_event);
        assert_eq!(result, break_event!(ControlResult::Continue));
        if let Some(menu) = testcase.menu {
            if let Some(last_item) = menu.menu_items.last() {
                assert_eq!(last_item.state, MenuState::Selected);
                assert_eq!(last_item.menu.as_ref().unwrap().state(), MenuState::Active);
            }
        }
    }

    #[test]
    fn is_selector() {
        let mut testcase = MenuItem::new("id", "testcase", 't');
        // enter key
        assert!(testcase.is_selector(&KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
        // ALT character
        assert!(!testcase.is_selector(&KeyEvent::new(KeyCode::Char('e'), KeyModifiers::ALT)));
        assert!(testcase.is_selector(&KeyEvent::new(KeyCode::Char('t'), KeyModifiers::ALT)));
        assert!(testcase.is_selector(&KeyEvent::new(KeyCode::Char('T'), KeyModifiers::ALT)));
        // selector character
        assert!(!testcase.is_selector(&KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE)));
        assert!(!testcase.is_selector(&KeyEvent::new(KeyCode::Char('T'), KeyModifiers::NONE)));
        testcase = testcase.with_char_select();
        assert!(testcase.is_selector(&KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE)));
        assert!(testcase.is_selector(&KeyEvent::new(KeyCode::Char('T'), KeyModifiers::NONE)));
    }
}
