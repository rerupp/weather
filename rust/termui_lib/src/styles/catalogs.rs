//! The terminal UI style catalog metadata.
//!
//! The hierarchy of metadata follows.
//!
//! * The metadata is organized as a collection of [CatalogType].
//! * All [CatalogType] variants in the collection have an associated [StyleCatalogs].
//! * All [StyleCatalogs] have a collection of [StyleCatalog] for all [ControlState] variants.
//! * All [StyleCatalog] have a collection of [CatalogStyle] for all [StyleId] variants.
//! * All [CatalogStyle] have an associated [Style] for a control.
//!
use super::{CatalogType, ControlState, StyleId, StyleTheme};
use ratatui::prelude::{Color, Modifier, Style};

/// The module that creates the default style catalogs.
mod bootstrap;

/// The module that can load style catalogs from a JSON document.
mod persistence;

/// The module that holds an in memory cache of the style catalogs.
mod store;

/// The public API that gets the style catalogs for a catalog type.
pub fn get_dark_styles3(catalog_type: CatalogType) -> &'static StyleCatalogs {
    store::get_dark_styles(catalog_type)
}

/// A catalog style identified by the [StyleId].
///
#[derive(Debug, PartialEq)]
struct CatalogStyle {
    /// The style identifier.
    style_id: StyleId,
    /// The control style.
    style: Style,
}
impl CatalogStyle {
    /// A catalog style always requires an identifier and a style.
    ///
    /// # Arguments
    ///
    /// * `style_id` identifies the style.
    /// * `style` is the associated [Style].
    ///
    fn new(style_id: StyleId, style: Style) -> Self {
        Self { style_id, style }
    }
}

/// The collection of [CatalogStyle] for a control state.
///
#[derive(Debug, PartialEq)]
pub struct StyleCatalog {
    /// The control state associated with the styles.
    pub control_state: ControlState,
    /// The collection of styles.
    catalog_styles: Vec<CatalogStyle>,
}
impl StyleCatalog {
    /// All style catalogs require a control state.
    ///
    /// # Arguments
    ///
    /// * `control_state` is the control state associated with the styles.
    ///
    fn new(control_state: ControlState) -> Self {
        Self { control_state, catalog_styles: vec![] }
    }
    #[cfg(test)]
    fn with_style(mut self, style_id: StyleId, style: Style) -> Self {
        self.catalog_styles.push(CatalogStyle { style_id, style });
        self
    }
    /// A builder method that adds a collection of styles to the catalog.
    ///
    /// # Arguments
    ///
    /// * `catalog_styles` is the collection of styles that will be added. There
    /// is no guarantee the resulting collection will not have duplicates.
    ///
    fn with_styles(mut self, mut catalogs_styles: Vec<CatalogStyle>) -> Self {
        self.catalog_styles.append(&mut catalogs_styles);
        self
    }
    /// Get a [Style] from the collection.
    ///
    /// # Arguments
    ///
    /// * `style_id` is the style identifier that will be returned.
    ///
    pub fn get(&self, style_id: StyleId) -> Style {
        self.catalog_styles[style_id as usize].style
    }
}

/// The collection of [StyleCatalog] for a [CatalogType].
pub struct StyleCatalogs {
    /// The catalog type identifier.
    catalog_type: CatalogType,
    /// The collection of style catalogs.
    catalogs: Vec<StyleCatalog>,
}
impl StyleCatalogs {
    /// All style catalogs require a control type.
    ///
    /// # Arguments
    ///
    /// * `catalog_type` is the catalog type associated with the styles.
    ///
    fn new(catalog_type: CatalogType) -> Self {
        Self { catalog_type, catalogs: vec![] }
    }
    #[cfg(test)]
    fn with_catalog(mut self, style_catalog: StyleCatalog) -> Self {
        self.catalogs.push(style_catalog);
        self
    }
    /// A builder method that adds a collection of catalog styles. There is no guarantee
    /// the resulting collection will not have duplicates.
    ///
    /// # Arguments
    ///
    /// * `catalogs` is the style catalogs collection that will be added.
    ///
    fn with_catalogs(mut self, mut catalogs: Vec<StyleCatalog>) -> Self {
        self.catalogs.append(&mut catalogs);
        self
    }
    /// Get a [StyleCatalog] reference from the catalog type collection.
    ///
    /// # Arguments
    ///
    /// * `control_state` identifies the control state catalog.
    ///
    pub fn get(&self, control_state: ControlState) -> &StyleCatalog {
        self.catalogs.get(control_state as usize).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use strum::IntoEnumIterator;
    use toolslib::{fmt::commafy, stopwatch::StopWatch};

    // #[test]
    #[allow(unused)]
    fn benchmark3() {
        // warm up the style catalogs
        get_dark_styles3(CatalogType::ButtonDialog);
        // the test results
        let mut catalogs_stopwatch = StopWatch::new();
        let mut catalogs_elapsed = Duration::default();
        let mut catalogs_lookups = 0usize;
        let mut lookups_stopwatch = StopWatch::new();
        let mut lookups_elapsed = Duration::default();
        let mut lookups = 0usize;
        // now run the benchmark
        for _ in 0..100 {
            for catalog_type in CatalogType::iter() {
                catalogs_stopwatch.start();
                let style_catalogs = get_dark_styles3(catalog_type);
                catalogs_elapsed += catalogs_stopwatch.elapsed();
                catalogs_lookups += 1;
                lookups_stopwatch.start();
                for control_state in ControlState::iter() {
                    let style_catalog = style_catalogs.get(control_state);
                    for style_id in StyleId::iter() {
                        style_catalog.get(style_id);
                        lookups += 1;
                    }
                }
                lookups_elapsed += lookups_stopwatch.elapsed();
            }
        }
        eprintln!(
            "Catalog lookups {}, elapsed {}us, {} ns/access",
            commafy(catalogs_lookups),
            commafy(catalogs_elapsed.as_micros()),
            catalogs_elapsed.as_nanos() / catalogs_lookups as u128
        );
        eprintln!(
            "Style lookups {}, elapsed {}us, {} ns/lookup",
            commafy(lookups),
            commafy(lookups_elapsed.as_micros()),
            lookups_elapsed.as_nanos() / lookups as u128
        );
        let overall_elapsed = catalogs_elapsed + lookups_elapsed;
        eprintln!(
            "Overall elapsed {}us, {} ns/lookup",
            commafy(overall_elapsed.as_micros()),
            overall_elapsed.as_nanos() / lookups as u128
        );
    }
}
