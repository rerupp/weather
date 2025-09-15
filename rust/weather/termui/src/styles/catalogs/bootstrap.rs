//! The hard coded style catalogs.
//!
//! I needed a starting point for the catalogs and this is it. Initially
//! I thought this would be thrown away however now I'm not so sure. This
//! makes it convenient if externalized style catalogs cannot be found or
//! have an error loading.
//!
use super::*;
use persistence::{StyleCatalogPDO, StyleCatalogsPDO, StyleThemePDO};

/// A helper that simply creates a [StyleCatalogPDO].
///
macro_rules! style_catalog {
    ($state:expr) => {
        StyleCatalogPDO::new($state)
    };
}

/// A helper that simply creates a [StyleCatalogsPDO].
///
macro_rules! style_catalogs {
    ($state:expr) => {
        StyleCatalogsPDO::new($state)
    };
}

/// The API used by the [persistence] module to get a bootstrapped version of the
/// dark theme.
pub fn bootstrap_dark_themes() -> StyleThemePDO {
    // modifiers
    let underline = Modifier::UNDERLINED;
    let italic = Modifier::ITALIC;
    let blink = Modifier::RAPID_BLINK;
    let bold = Modifier::BOLD;
    // the default style
    let default_style = Style::default()
        .fg(Color::Rgb(0xdc, 0xdf, 0xe4)) // MS Terminal 'One Half Dark' white
        .bg(Color::Rgb(0x28, 0x2c, 0x34)); // MS Terminal 'One Half Dark' background
    macro_rules! style {
        (bg=$bg:expr, attr=$attr:expr) => {
            default_style.bg($bg).add_modifier($attr).into()
        };
        (bg=$bg:expr) => {
            default_style.bg($bg).into()
        };
        (fg=$fg:expr) => {
            default_style.fg($fg).into()
        };
        (fg=$fg:expr, attr=$attr:expr) => {
            default_style.fg($fg).add_modifier($attr).into()
        };
        (fg=$fg:expr, bg=$bg:expr) => {
            default_style.fg($fg).bg($bg).into()
        };
        (attr=$attr:expr) => {
            default_style.add_modifier($attr).into()
        };
        (fg=$fg:expr, bg=$bg:expr, attr=$attr:expr) => {
            default_style.fg($fg).bg($bg).add_modifier($attr).into()
        };
    }
    // the style colors
    let dialog_border = Color::Rgb(0xad, 0xd8, 0xe6); // X11 light blue
    let dialog_title = Color::Rgb(0x87, 0xce, 0xfa); // X11 light sky blue
    let menubar_normal = Color::Rgb(0x64, 0x95, 0xed); // X11 corn flower blue
    let menubar_fg = Color::Rgb(0xd1, 0xe5, 0xf0); // rd bu10 color scheme #6
    let menubar_bg = Color::Rgb(0x21, 0x66, 0xac); // rd bu10 color scheme #9
    let report_view_header = Color::Rgb(0x87, 0xce, 0xeb); // // X11 sky blue
    let report_view_highlight = Color::Rgb(0x90, 0xee, 0x90); // X11 light green
    let highlight = Color::Rgb(0x90, 0xee, 0x90); // X11 light green
    let report_view_scrollbar = Color::Rgb(0xa4, 0xd3, 0xee); // X11 light blue 2
    let tab_dialog_border = Color::Rgb(0x7a, 0x8b, 0x8b); // X11 light cyan 4
    let tab_highlight_fg = Color::Rgb(0x10, 0x4e, 0x8b); // X11 dodger blue 4
    let tab_highlight_bg = Color::Rgb(0x87, 0xce, 0xfa); // X11 light sky blue
    let tab_text = Color::Rgb(0x87, 0xce, 0xfa); // X11 light sky blue

    // use the variants directly
    use CatalogType::*;
    use ControlState::*;
    use StyleId::*;
    StyleThemePDO::new(StyleTheme::Dark, default_style.into())
        .with_catalogs(
            style_catalogs!(ButtonDialog).with_catalog(
                style_catalog!(Active)
                    .with_override(ButtonBorder, style!(fg = dialog_border))
                    .with_override(DialogBorder, style!(fg = dialog_border))
                    .with_override(DialogTitle, style!(fg = dialog_title, attr = italic))
                    .with_override(LabelText, style!(fg = Color::LightBlue))
                    .with_override(LabelSelector, style!(fg = Color::Blue, attr = underline)),
            ),
        )
        .with_catalogs(
            style_catalogs!(CheckBoxGroup)
                .with_catalog(
                    style_catalog!(Active)
                        .with_override(LabelText, style!(fg = highlight))
                        .with_override(LabelSelector, style!(fg = highlight, attr = underline))
                        .with_override(GroupTitle, style!(fg = Color::Indexed(153)))
                        .with_override(Text, style!(attr = underline)),
                )
                .with_catalog(style_catalog!(Normal).with_override(LabelSelector, style!(attr = underline))),
        )
        .with_catalogs(
            style_catalogs!(EditGroup)
                .with_catalog(
                    style_catalog!(Active)
                        .with_override(LabelText, style!(fg = highlight))
                        .with_override(LabelSelector, style!(fg = highlight, attr = underline))
                        .with_override(GroupTitle, style!(fg = Color::Indexed(153)))
                        .with_override(Text, style!(attr = underline)),
                )
                .with_catalog(style_catalog!(Normal).with_override(LabelSelector, style!(attr = underline))),
        )
        .with_catalogs(
            style_catalogs!(MenuBar)
                .with_catalog(
                    style_catalog!(Active)
                        .with_override(LabelSelector, style!(fg = menubar_fg, bg = menubar_bg, attr = underline))
                        .with_override(LabelText, style!(fg = menubar_fg, bg = menubar_bg))
                        .with_override(DialogBorder, style!(fg = dialog_border))
                        .with_override(DialogTitle, style!(fg = dialog_title, attr = italic)),
                )
                .with_catalog(
                    style_catalog!(Normal)
                        .with_override(LabelSelector, style!(fg = menubar_normal, attr = underline))
                        .with_override(LabelText, style!(fg = menubar_normal)),
                ),
        )
        .with_catalogs(
            style_catalogs!(MenuDialog).with_catalog(
                style_catalog!(Active)
                    .with_override(DialogBorder, style!(fg = dialog_border))
                    .with_override(DialogTitle, style!(fg = Color::Blue))
                    .with_override(LabelSelector, style!(fg = Color::Blue, attr = underline))
                    .with_override(LabelText, style!(fg = Color::LightBlue)),
            ),
        )
        .with_catalogs(
            style_catalogs!(MessageDialog)
                .with_catalog(
                    style_catalog!(Error)
                        .with_override(ButtonBorder, style!(fg = Color::LightRed))
                        .with_override(DialogBorder, style!(fg = Color::Red))
                        .with_override(DialogTitle, style!(fg = Color::Red, attr = blink))
                        .with_override(LabelText, style!(fg = Color::LightRed))
                        .with_override(LabelSelector, style!(fg = Color::Red, attr = underline)),
                )
                .with_catalog(
                    style_catalog!(Warning)
                        .with_override(ButtonBorder, style!(fg = Color::LightYellow))
                        .with_override(DialogBorder, style!(fg = Color::Yellow))
                        .with_override(DialogTitle, style!(fg = Color::Yellow, attr = blink))
                        .with_override(LabelText, style!(fg = Color::LightYellow))
                        .with_override(LabelSelector, style!(fg = Color::Red, attr = underline)),
                ),
        )
        .with_catalogs(
            style_catalogs!(PopupMenu)
                .with_catalog(
                    style_catalog!(Active)
                        .with_override(DialogBorder, style!(fg = dialog_border))
                        .with_override(LabelSelector, style!(bg = Color::Indexed(27), attr = underline))
                        .with_override(LabelText, style!(bg = Color::Indexed(27))),
                )
                .with_catalog(style_catalog!(Normal).with_override(LabelSelector, style!(attr = underline))),
        )
        .with_catalogs(
            style_catalogs!(ProgressDialog).with_catalog(
                style_catalog!(Active)
                    .with_override(DialogBorder, style!(fg = dialog_border))
                    .with_override(Highlight, style!(fg = highlight)),
            ),
        )
        .with_catalogs(
            style_catalogs!(ReportView).with_catalog(
                style_catalog!(Active)
                    .with_override(Header, style!(fg = report_view_header, attr = bold))
                    .with_override(Highlight, style!(fg = report_view_highlight))
                    .with_override(Scrollbar, style!(fg = report_view_scrollbar)),
            ),
        )
        .with_catalogs(
            style_catalogs!(TabDialog)
                .with_catalog(
                    style_catalog!(Active)
                        .with_override(DialogBorder, style!(fg = tab_dialog_border))
                        .with_override(LabelSelector, style!(fg = tab_text, attr = underline))
                        .with_override(LabelText, style!(fg = tab_text))
                        .with_override(Highlight, style!(fg = tab_highlight_fg, bg = tab_highlight_bg)),
                )
                .with_catalog(
                    style_catalog!(Normal)
                        .with_override(DialogBorder, style!(fg = tab_dialog_border))
                        .with_override(LabelSelector, style!(attr = underline))
                        .with_override(LabelText, style!(fg = tab_text)),
                ),
        )
}
