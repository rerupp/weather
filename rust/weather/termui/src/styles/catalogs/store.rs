//! The style catalogs in memory cache.
//!
//! The cache is really a static singleton at this point. While it is limited it works
//! right now. Trying to make the cache high performant and available across threads
//! will take a bit of work but the ground work for that, if it is really required,
//! should be in place.
//!
use super::*;
use std::sync::OnceLock;

static DARK_STYLE_CATALOGS3: OnceLock<Vec<StyleCatalogs>> = OnceLock::new();

/// The public API that returns the catalog type style catalogs.
///
///# Arguments
///
///* `catalog_type` identifies what style catalogs to return.
pub fn get_dark_styles(catalog_type: CatalogType) -> &'static StyleCatalogs {
    // get_or_init guarantees the entire catalog is available
    let catalogs = DARK_STYLE_CATALOGS3.get_or_init(|| {
        let (default_style, mut style_catalogs) = persistence::load_dark_theme();
        assemble::style_catalogs(&mut style_catalogs, default_style);
        style_catalogs
    });
    &catalogs[catalog_type as usize]
}

/// Assembles style catalogs into an ordered collection.
mod assemble {
    //! Assemble a collection of style catalogs.
    //!
    //! The catalogs are examined, removing duplicate entries and ensuring a style exist for
    //! each [CatalogType], [ControlState], and [StyleId] variant. The catalogs are ordered
    //! allowing them to be indexed by each of the variants.
    use super::*;
    use strum::IntoEnumIterator;

    /// The public API that assembles a collection of style catalogs.
    ///
    /// # Arguments
    ///
    /// * `catalogs` is the collection that will be assembled.
    /// * `default_style` is the style used when a variant entry is missing from the collection.
    ///
    pub fn style_catalogs(catalogs: &mut Vec<StyleCatalogs>, default_style: Style) {
        // make sure there are no duplicate style catalogs (be predictable with the order of duplicates)
        catalogs.sort_by(|lhs, rhs| lhs.catalog_type.cmp(&rhs.catalog_type));
        catalogs.dedup_by(|lhs, rhs| lhs.catalog_type == rhs.catalog_type);
        // now fill in the blanks
        for catalog_type in CatalogType::iter() {
            match catalogs.iter_mut().find(|style_catalogs| style_catalogs.catalog_type == catalog_type) {
                None => catalogs.push(default_style_catalogs(catalog_type, default_style)),
                Some(style_catalogs) => assemble_style_catalogs(style_catalogs, default_style),
            }
        }
        // make sure the catalogs are in catalog type order
        catalogs.sort_unstable_by(|lhs, rhs| lhs.catalog_type.cmp(&rhs.catalog_type));
    }

    /// Walk the style catalogs making sure all [ControlState] and [StyleId] variants have an
    /// associated [Style].
    ///
    /// # Arguments
    ///
    /// * `style_catalogs` are the style catalogs that will be examined.
    /// * `default_style` is what will be used if a [Style] is missing.
    ///
    fn assemble_style_catalogs(style_catalogs: &mut StyleCatalogs, default_style: Style) {
        // make sure there are no duplicate catalog types (be predictable with the order of duplicates)
        style_catalogs.catalogs.sort_by(|lhs, rhs| lhs.control_state.cmp(&rhs.control_state));
        style_catalogs.catalogs.dedup_by(|lhs, rhs| lhs.control_state == rhs.control_state);
        // make sure all the control states are present
        for control_state in ControlState::iter() {
            match style_catalogs.catalogs.iter_mut().find(|style_catalog| style_catalog.control_state == control_state)
            {
                None => style_catalogs.catalogs.push(default_style_catalog(control_state, default_style)),
                Some(style_catalog) => assemble_style_catalog(style_catalog, default_style),
            }
        }
        // now make sure the style catalogs are in order
        style_catalogs.catalogs.sort_unstable_by(|lhs, rhs| lhs.control_state.cmp(&rhs.control_state));
    }

    /// Walk the style catalog making sure all [StyleId] variants have an associated [Style].
    ///
    /// # Arguments
    ///
    /// * `style_catalog` is the style catalog that will be examined.
    /// * `default_style` is what will be used if a [Style] is missing.
    ///
    fn assemble_style_catalog(style_catalog: &mut StyleCatalog, default_style: Style) {
        // make sure there are no duplicate styles (be predictable with the order of duplicates)
        style_catalog.catalog_styles.sort_by(|lhs, rhs| lhs.style_id.cmp(&rhs.style_id));
        style_catalog.catalog_styles.dedup_by(|lhs, rhs| {
            let is_duplicate = lhs.style_id == rhs.style_id;
            if is_duplicate {
                log::debug!("removing duplicate style id {} in {}", lhs.style_id, style_catalog.control_state);
            }
            is_duplicate
        });
        // make sure all the styles are there
        for style_id in StyleId::iter() {
            if style_catalog.catalog_styles.iter().find(|catalog_style| catalog_style.style_id == style_id).is_none() {
                log::trace!("adding style id {} in {}", style_id, style_catalog.control_state);
                style_catalog.catalog_styles.push(CatalogStyle::new(style_id, default_style));
            }
        }
        // make sure the styles are in order
        style_catalog.catalog_styles.sort_unstable_by(|lhs, rhs| lhs.style_id.cmp(&rhs.style_id));
    }

    /// Creates style catalogs with all styles reflecting the default style passed in.
    ///
    /// # Arguments
    ///
    /// * `catalogs_type` is the type of style catalog to create.
    /// * `default_style` is what will be used if a [Style] is missing.
    fn default_style_catalogs(catalog_type: CatalogType, default_style: Style) -> StyleCatalogs {
        log::debug!("adding default style catalogs for {}", catalog_type);
        StyleCatalogs::new(catalog_type).with_catalogs(
            ControlState::iter().map(|control_state| default_style_catalog(control_state, default_style)).collect(),
        )
    }

    /// Creates style catalog with all styles reflecting the default style passed in.
    ///
    /// # Arguments
    ///
    /// * `control_state` identifies the type of style catalog to create.
    /// * `default_style` is what will be used if a [Style] is missing.
    fn default_style_catalog(control_state: ControlState, default_style: Style) -> StyleCatalog {
        log::trace!("Adding default style catalog for {}", control_state);
        StyleCatalog::new(control_state).with_styles(default_catalog_styles(default_style))
    }

    /// Create a default catalog style.
    ///
    /// # Arguments
    ///
    /// * `default_style` is used to initialize the catalog style..
    fn default_catalog_styles(default_style: Style) -> Vec<CatalogStyle> {
        log::trace!("adding default catalog styles");
        StyleId::iter().map(|style_id| CatalogStyle::new(style_id, default_style)).collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn default_catalog_styles() {
            let default_style = Style::default().fg(Color::Gray);
            let testcase = super::default_catalog_styles(default_style);
            for style_id in StyleId::iter() {
                assert_eq!(testcase[style_id as usize].style_id, style_id);
                assert_eq!(testcase[style_id as usize].style, default_style, "{}", style_id)
            }
        }

        #[test]
        fn default_style_catalog() {
            let control_state = ControlState::Active;
            let default_style = Style::default().bg(Color::Gray);
            let testcase = super::default_style_catalog(control_state, default_style);
            assert_eq!(testcase.control_state, control_state);
            for style_id in StyleId::iter() {
                assert_eq!(testcase.catalog_styles[style_id as usize].style_id, style_id);
                assert_eq!(testcase.catalog_styles[style_id as usize].style, default_style, "{}", style_id)
            }
        }

        #[test]
        fn default_style_catalogs() {
            let catalog_type = CatalogType::EditGroup;
            let default_style = Style::default().add_modifier(Modifier::ITALIC);
            let testcase = super::default_style_catalogs(catalog_type, default_style);
            assert_eq!(testcase.catalog_type, catalog_type);
            for control_state in ControlState::iter() {
                let style_catalog = &testcase.catalogs[control_state as usize];
                assert_eq!(style_catalog.control_state, control_state);
                for style_id in StyleId::iter() {
                    assert_eq!(style_catalog.catalog_styles[style_id as usize].style_id, style_id, "{}", control_state);
                    assert_eq!(
                        style_catalog.catalog_styles[style_id as usize].style, default_style,
                        "{} {}",
                        control_state, style_id
                    )
                }
            }
        }

        #[test]
        fn assemble_style_catalog() {
            let default_style = Style::default().bg(Color::Black);
            let header_style = Style::default().fg(Color::Blue);
            let title_style = Style::default().add_modifier(Modifier::UNDERLINED);
            let mut testcase = StyleCatalog::new(ControlState::Normal)
                .with_style(StyleId::Header, header_style)
                .with_style(StyleId::Header, default_style)
                .with_style(StyleId::DialogTitle, title_style);
            super::assemble_style_catalog(&mut testcase, default_style);
            for style_id in StyleId::iter() {
                match style_id {
                    StyleId::Header => assert_eq!(testcase.get(style_id), header_style, "{}", style_id),
                    StyleId::DialogTitle => assert_eq!(testcase.get(style_id), title_style, "{}", style_id),
                    _ => assert_eq!(testcase.get(style_id), default_style, "{}", style_id),
                }
            }
        }

        #[test]
        fn assemble_style_catalogs() {
            let default_style = Style::default().fg(Color::Black);
            let highlight_style = Style::default().fg(Color::Yellow);
            let text_style = Style::default().fg(Color::White);
            let mut style_catalogs = StyleCatalogs::new(CatalogType::ButtonDialog)
                .with_catalog(StyleCatalog::new(ControlState::Active).with_style(StyleId::Highlight, highlight_style))
                .with_catalog(StyleCatalog::new(ControlState::Active).with_style(StyleId::Highlight, default_style))
                .with_catalog(StyleCatalog::new(ControlState::Normal).with_style(StyleId::Text, text_style));
            super::assemble_style_catalogs(&mut style_catalogs, default_style);
            for control_state in ControlState::iter() {
                for style_id in StyleId::iter() {
                    let style = style_catalogs.get(control_state).get(style_id);
                    match (control_state, style_id) {
                        (ControlState::Active, StyleId::Highlight) => assert_eq!(style, highlight_style),
                        (ControlState::Normal, StyleId::Text) => assert_eq!(style, text_style),
                        _ => assert_eq!(style, default_style, "{} {}", control_state, style_id),
                    }
                }
            }
        }

        #[test]
        fn style_catalogs() {
            let default_style = Style::default().fg(Color::Blue);
            let header_style = Style::default().add_modifier(Modifier::ITALIC);
            let text_style = Style::default().fg(Color::Red);
            use CatalogType::*;
            use ControlState::*;
            use StyleId::*;
            let mut testcase = vec![
                StyleCatalogs::new(TabDialog).with_catalog(StyleCatalog::new(Error).with_style(Text, text_style)),
                StyleCatalogs::new(ButtonDialog)
                    .with_catalog(StyleCatalog::new(Normal).with_style(Header, header_style)),
                StyleCatalogs::new(ButtonDialog)
                    .with_catalog(StyleCatalog::new(Active).with_style(Header, header_style)),
            ];
            super::style_catalogs(&mut testcase, default_style);
            for catalog_type in CatalogType::iter() {
                let style_catalogs = testcase.get(catalog_type as usize).unwrap();
                assert_eq!(style_catalogs.catalog_type, catalog_type);
                for control_state in ControlState::iter() {
                    let style_catalog = style_catalogs.get(control_state);
                    assert_eq!(style_catalog.control_state, control_state, "{}", catalog_type);
                    for style_id in StyleId::iter() {
                        let style = style_catalog.get(style_id);
                        match (catalog_type, control_state, style_id) {
                            (ButtonDialog, Normal, Header) => {
                                assert_eq!(style, header_style, "{} {} {}", catalog_type, control_state, style_id);
                            }
                            (TabDialog, Error, Text) => {
                                assert_eq!(style, text_style, "{} {} {}", catalog_type, control_state, style_id);
                            }
                            _ => assert_eq!(style, default_style, "{} {} {}", catalog_type, control_state, style_id),
                        }
                    }
                }
            }
        }
    }
}
