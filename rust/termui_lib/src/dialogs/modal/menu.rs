//! The TUI menu dialog.
//! This modal dialog combines a [DialogWindow] and a [Menubar]. The `MenuBar` is drawn at the
/// top of the screen area and the `DialogWindow` below it.
///
use super::*;
use menus::Menubar;
use ratatui::symbols::border::Set;
use std::fmt::Debug;
use styles::ActiveNormalStyles;

/// The menu dialog.
///
#[derive(Debug)]
pub struct MenuDialog<T: DialogWindow + Debug> {
    /// The frame of the dialog.
    frame: DialogFrame,
    // /// The dialog menu.
    // menubar: Menubar,
    /// The dialog menu.
    menubar: Menubar,
    /// Indicates the menu should be drawn in the center of the screen area.
    center_screen: bool,
    /// Draw a separator line between the menu and window.
    menu_separator: bool,
    /// The window managed by the dialog.
    window: T,
    /// The menu dialog style catalog type. This will always be [CatalogType::MenuDialog].
    pub catalog_type: CatalogType,
}
impl<T: DialogWindow + Debug> MenuDialog<T> {
    /// Create a new instance of the menu dialog.
    ///
    /// # Arguments
    /// - `menubar` is the dialog menu.
    /// - `window` is the window that will be managed.
    ///
    pub fn new(menubar: Menubar, window: T) -> Self {
        Self {
            frame: Default::default(),
            menubar,
            center_screen: false,
            menu_separator: false,
            window,
            catalog_type: CatalogType::MenuDialog,
        }
    }
    /// A builder method that configures the dialog to draw itself centered in the screen area.
    ///
    pub fn with_center_screen(mut self) -> Self {
        self.center_screen = true;
        self
    }
    /// A builder method that configures the dialog to draw a separator between the menu and window.
    ///
    pub fn with_menu_separator(mut self) -> Self {
        self.menu_separator = true;
        self
    }
    /// A builder method that configures the dialog to draw a surrounding border.
    ///
    pub fn with_border(mut self) -> Self {
        self.frame.draw_border = true;
        self
    }
    /// Force the menu to reset it state.
    pub fn reset_menu(&mut self) {
        self.menubar.reset();
    }
    /// Create a dialog message.
    ///
    /// # Arguments
    ///
    /// - `style` determines how the message dialog will be drawn.
    /// - `message` is the text that will be displayed.
    ///
    pub fn set_message(&mut self, style: MessageStyle, message: impl ToString) {
        self.frame.message.replace(MessageDialog::new(style, message));
    }
    /// Get a reference to the managed window.
    ///
    pub fn win(&self) -> &T {
        &self.window
    }
    /// Get a mutable reference to the managed window.
    ///
    pub fn win_mut(&mut self) -> &mut T {
        &mut self.window
    }
    /// Consume a key pressed event. The event will be passed to the menu if it is currently
    /// active otherwise it is passed on to the window, frame, and then menu. [ControlFlow::Continue] will
    /// be returned if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    pub fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<DialogResult> {
        // the menu will either consume the event or continue
        macro_rules! menu_key_pressed {
            () => {
                match self.menubar.key_pressed(key_event) {
                    ControlFlow::Break(ControlResult::Selected(id)) => break_event!(DialogResult::Selected(id)),
                    ControlFlow::Break(ControlResult::Continue) => break_event!(DialogResult::Continue),
                    _ => ControlFlow::Continue(()),
                }
            };
        }
        log_key_pressed!("MenuDialog");
        match self.menubar.is_active() {
            true => menu_key_pressed!()?,
            false => {
                self.window.key_pressed(key_event)?;
                menu_key_pressed!()?;
            }
        }
        if ControlFlow::Break(DialogResult::Cancel) == self.frame.key_pressed(key_event) {
            match self.menubar.is_active() {
                false => break_event!(DialogResult::Cancel)?,
                true => {
                    self.menubar.reset();
                    break_event!(DialogResult::Continue)?;
                }
            }
        }
        beep();
        ControlFlow::Continue(())
    }
    /// Draw the dialog on the terminal screen. The order of rendering is the frame, the window,
    /// the message dialog (if set). followed by the menu. The screen position of the cursor will
    /// be returned if available.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the dialog can be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    pub fn render(&self, area: Rect, buffer: &mut Buffer) -> Option<Position> {
        let (frame_area, window_area, menu_area) = self.dialog_areas(area);
        log_render!("MenuDialog");
        // render the frame and get the area available for rendering
        self.frame.render(None, frame_area, buffer, self.catalog_type.get_styles(ControlState::Active));
        // render the window first to allow the menu to overlay it
        let mut coord = self.window.render(window_area, buffer);
        let message = self.frame.message.is_some();
        let menu_active = self.menubar.is_active();
        debug_assert!(!(message && menu_active), "frame message and menu both active...\n{:#?}", self);
        if message {
            let message_dialog = self.frame.message.as_ref().unwrap();
            if let Some(message_coord) = message_dialog.render(area, buffer) {
                coord.replace(message_coord);
            }
        }
        // now show the menubar
        let menubar_styles = ActiveNormalStyles::new(self.menubar.catalog_type);
        if self.menu_separator {
            let separator_area = inner_rect(menu_area, (0, 1), (0, 2));
            let border_set = match self.frame.draw_border {
                true => Set {
                    top_left: symbols::line::VERTICAL_LEFT,
                    top_right: symbols::line::VERTICAL_RIGHT,
                    ..Default::default()
                },
                false => Set {
                    top_left: symbols::line::HORIZONTAL,
                    top_right: symbols::line::HORIZONTAL,
                    ..Default::default()
                },
            };
            Block::new()
                .borders(Borders::ALL)
                .border_set(border_set)
                .border_style(menubar_styles.active.get(StyleId::DialogBorder))
                .render(separator_area, buffer);
        }
        if let Some(menubar_coord) = self.menubar.render(menu_area, buffer, menubar_styles) {
            coord.replace(menubar_coord);
        }
        coord
    }
    /// Create the drawing areas for the frame, menu, and window.
    ///
    /// # Arguments
    ///
    /// - `area` is where the dialog will be drawn.
    ///
    fn dialog_areas(&self, area: Rect) -> (Rect, Rect, Rect) {
        // set up the frame area
        let window_size = self.window.size();
        let menu_size = self.menubar.size();
        let frame_area = match self.center_screen {
            false => area,
            true => {
                // the width needs to account for the border
                let width = 2u16.saturating_add(cmp::max(window_size.width, menu_size.width));
                // the height needs to account for the border and if applicable menu/window separator
                let height = match self.menu_separator {
                    true => 3u16,
                    false => 2u16,
                }
                .saturating_add(window_size.height)
                .saturating_add(menu_size.height);
                center_rect!(area, [cmp::min(width, area.width), cmp::min(height, area.height)])
            }
        };
        // let render_area = inner_rect(frame_area, (1, 1), (-1, -1));
        let render_area = match self.frame.draw_border {
            true => inner_rect(frame_area, (1, 1), (-1, -1)),
            false => area,
        };
        // the menu area needs to span the available screen to allow submenus to render
        let menu_area = inner_rect(render_area, (1, 0), (-1, 0));
        // let menu_area = inner_rect(render_area, (0, 0), (0, menu_size.height as i32));
        let window_top = match self.menu_separator {
            true => menu_size.height + 1,
            false => menu_size.height,
        } as i32;
        let window_area = inner_rect(render_area, (0, window_top), (0, 0));
        // the menu can overlap into the window area but should not overlay the dialog frame
        (frame_area, window_area, menu_area)
    }
}
