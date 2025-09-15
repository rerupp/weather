//! The `ratatui` `Style` catalogs used to render TUI controls.

pub use catalogs::{StyleCatalog, StyleCatalogs};
mod catalogs;

use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// The collection of styles for types that need to render active and normal controls.
///
#[derive(Debug, Copy, Clone)]
pub struct ActiveNormalStyles<'s> {
    /// The active control style catalog.
    pub active: &'s StyleCatalog,
    /// The non-active control style catalog.
    pub normal: &'s StyleCatalog,
}
impl<'s> ActiveNormalStyles<'s> {
    /// Create the catalogs for a catalog type.
    ///
    /// # Arguments
    ///
    /// * `catalog_type` identifies which style catalogs to use.
    ///
    pub fn new(catalog_type: CatalogType) -> Self {
        let style_catalogs = catalogs::get_dark_styles3(catalog_type);
        Self { active: style_catalogs.get(ControlState::Active), normal: style_catalogs.get(ControlState::Normal) }
    }
    /// Create the catalogs for a catalog type.
    ///
    /// # Arguments
    ///
    /// * `catalog_type` identifies which style catalogs to use.
    /// * `control_state` identifies what control state should be considered active.
    ///
    pub fn with_active_style(catalog_type: CatalogType, control_state: ControlState) -> Self {
        let style_catalogs = catalogs::get_dark_styles3(catalog_type);
        Self { active: style_catalogs.get(control_state), normal: style_catalogs.get(ControlState::Normal) }
    }
}

/// The render states of a control.
///
#[derive(Copy, Clone, Debug, Deserialize, EnumIter, Eq, Ord, PartialOrd, PartialEq, Serialize)]
pub enum ControlState {
    /// The controls state is active.
    Active,
    /// The control state is in an error condition.
    Error,
    /// The control state is not active.
    Normal,
    /// The control state is in a warning condition.
    Warning,
}
impl std::fmt::Display for ControlState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// The `ratatui Style` catalog types.
///
#[derive(Copy, Clone, Debug, Deserialize, EnumIter, Eq, Ord, PartialOrd, PartialEq, Serialize)]
pub enum CatalogType {
    /// A button dialog.
    ButtonDialog,
    /// A checkbox group.
    CheckBoxGroup,
    /// An edit group.
    EditGroup,
    /// A menubar.
    MenuBar,
    /// A menu Dialog.
    MenuDialog,
    /// A message Dialog.
    MessageDialog,
    /// A popup menu.
    PopupMenu,
    /// A progress dialog.
    ProgressDialog,
    /// A report viewer.
    ReportView,
    /// A tab dialog.
    TabDialog,
}
impl CatalogType {
    /// I guess this could be considered an anti-pattern but it makes sense to me a catalog type
    /// should be able to retrieve its own style catalog
    ///
    /// # Arguments
    ///
    /// * `control_state` identifies the style catalog.
    ///
    pub fn get_styles(&self, control_state: ControlState) -> &'static StyleCatalog {
        catalogs::get_dark_styles3(*self).get(control_state)
    }
}
impl std::fmt::Display for CatalogType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// The various style identifiers.
///
#[derive(Clone, Copy, Debug, EnumIter, Eq, Ord, PartialOrd, PartialEq, Deserialize, Serialize)]
pub enum StyleId {
    /// The style identifier for the border of a button.
    ButtonBorder,
    /// The style identifier for the border of a dialog.
    DialogBorder,
    /// The style identifier for a dialog title.
    DialogTitle,
    /// The style identifier for the title of a group of controls.
    GroupTitle,
    /// The style identifier for control header.
    Header,
    /// The style identifier for a control that should be highlighted.
    Highlight,
    /// The style identifier for a control selector.
    LabelSelector,
    /// The style identifier for a labels text.
    LabelText,
    /// The style identifier for the screen area used by a control.
    Screen,
    /// The style identifier for a control's scrollbar.
    Scrollbar,
    /// The style identifier for text controls.
    Text,
}
impl std::fmt::Display for StyleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// The different version of style catalogs available.
///
#[derive(Debug, Serialize, Deserialize)]
pub enum StyleTheme {
    /// Indicates the style catalogs follow the dark theme.
    Dark,
}
impl std::fmt::Display for StyleTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
