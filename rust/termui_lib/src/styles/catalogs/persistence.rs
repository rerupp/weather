//! The persistent data object (PDO) layer for style catalogs in the application stack.
//!
//! This is the collection of objects that map the [styles](crate::styles) structures into something that
//! can be serialized and deserialized from some source. Currently, the persistence source
//! needs to be a JSON document but that's a Rick decision not a requirement. Eventually it
//! will probably be YAML since I find the TOML format, for this type of document, to be
//! horrendous to read.
//!
//! One of the patterns I really like in Rust is the [From] and [Into] traits. It allows
//! the mapping between types to be well-defined. In Java land you would create the PDO and
//! invent a naming pattern like `into_something()` or `from_something()` to map between types.
//! You still need to code the traits in Rust but once that's done you get both a `from()` method
//! and an `into()` method. The `into()` method is nice because the compiler looks for that
//! trait as a way of converting some type into another type.
//!
use super::{
    bootstrap, CatalogStyle, CatalogType, Color, ControlState, Modifier, Style, StyleCatalog, StyleCatalogs, StyleId,
    StyleTheme,
};
use serde::{Deserialize, Serialize};

/// The API used by the [styles](crate::styles::catalogs::store) module to create the memory cache of styles.
///
pub fn load_dark_theme() -> (Style, Vec<StyleCatalogs>) {
    let theme = bootstrap::bootstrap_dark_themes();
    let default_style = Style::from(theme.default_style);
    let style_catalogs = theme
        .style_catalogs
        .into_iter()
        .map(|catalogs| {
            StyleCatalogs::new(catalogs.catalog_type).with_catalogs(
                catalogs
                    .style_catalogs
                    .into_iter()
                    .map(|catalog| {
                        StyleCatalog::new(catalog.control_state).with_styles(
                            catalog.style_overrides.into_iter().map(|style| CatalogStyle::from(style)).collect(),
                        )
                    })
                    .collect::<Vec<StyleCatalog>>(),
            )
        })
        .collect::<Vec<StyleCatalogs>>();
    (default_style, style_catalogs)
}

/// The PDO that represents a `ratatui Style` object.
///
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StylePDO {
    /// The style foreground.
    fg: Color,
    /// The style background.
    bg: Color,
    /// The style modifiers.
    attr: Modifier,
}
impl StylePDO {
    /// A style PDO requires all members to be filled out.
    ///
    /// # Arguments
    ///
    /// * `
    pub fn new(fg: Color, bg: Color, attr: Modifier) -> Self {
        Self { fg, bg, attr }
    }
}
impl From<Style> for StylePDO {
    fn from(style: Style) -> Self {
        StylePDO::new(style.fg.unwrap_or(Color::Reset), style.bg.unwrap_or(Color::Reset), style.add_modifier)
    }
}
impl From<StylePDO> for Style {
    fn from(style_pdo: StylePDO) -> Self {
        let mut style = Style::default().add_modifier(style_pdo.attr);
        if style_pdo.fg != Color::Reset {
            style.fg.replace(style_pdo.fg);
        }
        if style_pdo.bg != Color::Reset {
            style.bg.replace(style_pdo.bg);
        }
        style
    }
}

/// The PDO that represents a [CatalogStyle] structure.
///
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StyleOverridePDO {
    /// The style identifier.
    #[serde(rename = "style-id")]
    style_id: StyleId,
    /// The `ratatui` [Style].
    style: StylePDO,
}
impl StyleOverridePDO {
    /// A style override requires both the style identifier and style definition.
    ///
    /// # Arguments
    ///
    /// * `style_id` is the style identifier.
    /// * `style` is the style definition.
    ///
    pub fn new(style_id: StyleId, style: StylePDO) -> Self {
        Self { style_id, style }
    }
}
impl From<&CatalogStyle> for StyleOverridePDO {
    fn from(style_override: &CatalogStyle) -> Self {
        Self::new(style_override.style_id, StylePDO::from(style_override.style))
    }
}
impl From<StyleOverridePDO> for CatalogStyle {
    fn from(style_override: StyleOverridePDO) -> Self {
        CatalogStyle { style_id: style_override.style_id, style: style_override.style.into() }
    }
}

/// The PDO that represents a [StyleCatalog] structure.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct StyleCatalogPDO {
    /// The control state associate with the collection of styles.
    #[serde(rename = "control-state")]
    control_state: ControlState,
    /// The collection of styles.
    style_overrides: Vec<StyleOverridePDO>,
}
impl StyleCatalogPDO {
    /// All style catalogs require a control state.
    ///
    /// # Arguments
    ///
    /// * `control_state` is the catalog control state.
    pub fn new(control_state: ControlState) -> Self {
        Self { control_state, style_overrides: vec![] }
    }
    /// Add a catalog style to the collection.
    ///
    /// # Arguments
    ///
    /// * `style_id` is the style identifier. There is no guarantee the resulting collection will not have
    ///    duplicates.
    /// * `style` is the style definition.
    pub fn with_override(mut self, style_id: StyleId, style: StylePDO) -> Self {
        self.style_overrides.push(StyleOverridePDO { style_id, style });
        self
    }
    /// Add a collection of catalog styles to the collection.
    ///
    /// # Arguments
    ///
    /// * `overrides` are the styles that will be added to the collection. There
    /// is no guarantee the resulting collection will not have duplicates.
    pub fn with_overrides(mut self, mut overrides: Vec<StyleOverridePDO>) -> Self {
        self.style_overrides.append(&mut overrides);
        self
    }
}
impl From<StyleCatalogPDO> for StyleCatalog {
    fn from(style_catalog: StyleCatalogPDO) -> Self {
        StyleCatalog::new(style_catalog.control_state).with_styles(
            style_catalog.style_overrides.into_iter().map(|style_override| style_override.into()).collect(),
        )
    }
}

/// The PDO that represents a [StyleCatalogs] structure.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct StyleCatalogsPDO {
    /// The catalog type associated with the collection of style catalogs.
    #[serde(rename = "catalog-type")]
    catalog_type: CatalogType,
    /// The collection of style catalogs.
    style_catalogs: Vec<StyleCatalogPDO>,
}
impl StyleCatalogsPDO {
    /// All style catalogs require a catalog type.
    pub fn new(catalog_type: CatalogType) -> Self {
        Self { catalog_type, style_catalogs: vec![] }
    }
    /// Add a style catalog to the collection.
    ///
    /// # Arguments
    ///
    /// * `style_catalog` is the style catalog that will be added. There is no guarantee the resulting
    /// collection will not have duplicates.
    ///
    pub fn with_catalog(mut self, style_catalog: StyleCatalogPDO) -> Self {
        self.style_catalogs.push(style_catalog);
        self
    }
    /// Add a collection of style catalogs to the collection.
    ///
    /// # Arguments
    ///
    /// * `catalogs` is the collection of style catalogs to add. There is no guarantee the resulting
    /// collection will not have duplicates.
    pub fn with_catalogs(mut self, mut catalogs: Vec<StyleCatalogPDO>) -> Self {
        self.style_catalogs.append(&mut catalogs);
        self
    }
}
impl From<StyleCatalogsPDO> for StyleCatalogs {
    fn from(style_catalogs: StyleCatalogsPDO) -> Self {
        StyleCatalogs::new(style_catalogs.catalog_type).with_catalogs(
            style_catalogs.style_catalogs.into_iter().map(|style_catalog| style_catalog.into()).collect(),
        )
    }
}

/// The PDO object the holds a style catalogs collection for some [StyleTheme] variant.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct StyleThemePDO {
    /// The style theme identifier.
    #[serde(rename = "theme")]
    style_theme: StyleTheme,
    /// Each theme has a default style associated with it. This style will be used if a corresponding
    /// control state, style catalog, or style identifier cannot be found in a collection.
    #[serde(rename = "default-style")]
    default_style: StylePDO,
    /// The collection of style catalogs defined for the theme.
    #[serde(rename = "style-catalogs")]
    style_catalogs: Vec<StyleCatalogsPDO>,
}
impl StyleThemePDO {
    /// A theme requires a theme identifier and a default style definition.
    pub fn new(style_theme: StyleTheme, default_style: StylePDO) -> Self {
        Self { style_theme, default_style, style_catalogs: vec![] }
    }
    /// Add a style catalog to the theme. There is no guarantee the resulting collection will
    /// not have duplicates.
    pub fn with_catalogs(mut self, style_catalogs: StyleCatalogsPDO) -> Self {
        self.style_catalogs.push(style_catalogs);
        self
    }
}
impl From<StyleThemePDO> for Vec<StyleCatalogs> {
    fn from(style_theme: StyleThemePDO) -> Self {
        style_theme.style_catalogs.into_iter().map(|catalogs| catalogs.into()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style() {
        let style = Style::default();
        assert_eq!(style, Style::from(StylePDO::from(style)));
        let style = Style::default().fg(Color::Gray);
        assert_eq!(style, Style::from(StylePDO::from(style)));
        let style = Style::default().bg(Color::Gray);
        assert_eq!(style, Style::from(StylePDO::from(style)));
        let style = Style::default().add_modifier(Modifier::ITALIC);
        assert_eq!(style, Style::from(StylePDO::from(style)));
        let style = Style::default()
            .fg(Color::Yellow)
            .bg(Color::Gray)
            .add_modifier(Modifier::UNDERLINED | Modifier::ITALIC | Modifier::RAPID_BLINK);
        let testcase = StylePDO::from(style);
        assert_eq!(Style::from(testcase), style);
    }

    #[test]
    fn style_override() {
        let style_override = CatalogStyle { style_id: StyleId::Text, style: Style::default().fg(Color::Black) };
        let testcase = StyleOverridePDO::from(&style_override);
        assert_eq!(CatalogStyle::from(testcase), style_override);
    }
}
