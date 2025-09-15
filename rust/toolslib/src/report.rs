#![allow(unused)]

/// The second version of the report writer
use super::*;
use std::cmp;

/// The alignment of text rows column.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum CellAlignment {
    /// Cell data will be left aligned.
    Left,
    /// Cell data will be center aligned.
    Center,
    /// Cell data will be right aligned.
    Right,
    /// This indicates the cell does not have a layout and should be output as is.
    #[default]
    None,
}
impl CellAlignment {
    /// Queries if the variant is [None](Self::None).
    pub fn is_none(&self) -> bool {
        self == &Self::None
    }
}

/// The category of data at a rows and column
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CellType {
    /// The rows column is header text.
    Header,
    /// The rows column is separator text.
    Separator,
    /// The rows column is text.
    Text,
    /// The rows column should be left as is.
    Plain,
}

/// The description of data for a rows column.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CellLayout {
    /// The alignment of cell data.
    alignment: CellAlignment,
    /// The column width of cell data.
    width: usize,
    /// Cell data is fixed width.
    fixed: bool,
    /// Cell data will be repeated to fill the cell width.
    fill: bool,
    // /// The number of data cells this definition applies too.
    // spans: usize,
}
impl CellLayout {
    /// Create a description of the text at a row's column.
    ///
    /// # Arguments
    ///
    /// - `alignment` defines the text position within the row's column.
    ///
    pub fn new(alignment: CellAlignment) -> Self {
        Self { alignment, width: 0, fixed: false, fill: false }
    }
    /// Set the minimum width of text at a row's column.
    ///
    /// # Arguments
    ///
    /// - `width` is the minimum width of a row's column.
    ///
    pub fn with_width(mut self, width: usize) -> Self {
        if !self.alignment.is_none() {
            self.width = width;
            self.fixed = false;
        }
        self
    }
    /// Set row's column tp a fixed width.
    ///
    /// # Arguments
    ///
    /// - `width` sets the width of the row's column.
    ///
    pub fn with_fixed(mut self, width: usize) -> Self {
        if !self.alignment.is_none() {
            self.width = width;
            self.fixed = true;
        }
        self
    }
    /// Forces text to be repeated until it fills a row's column width.
    ///
    pub fn with_fill(mut self) -> Self {
        if !self.alignment.is_none() {
            self.fill = true;
        }
        self
    }
    /// Get the width of a row's column.
    ///
    pub fn width(&self) -> usize {
        self.width
    }
    /// Format text according to the cell layout.
    ///
    /// # Arguments
    ///
    /// - `text` is the string that will be formatted.
    ///
    pub fn format(&self, text: impl ToString) -> String {
        let mut cell_text = text.to_string();
        let text_len = cell_text.len();
        if self.fill && text_len > 0 {
            let repeat = (self.width / text_len) + 1;
            cell_text = cell_text.repeat(repeat)[0..self.width].to_string();
        };
        let cell_width = self.width;
        match self.alignment {
            CellAlignment::Left | CellAlignment::None => format!("{cell_text:<cell_width$}"),
            CellAlignment::Center => format!("{cell_text:^cell_width$}"),
            CellAlignment::Right => format!("{cell_text:>cell_width$}"),
        }
    }
}

/// A [row's](SheetRow) column data and layout.
#[derive(Debug, PartialEq)]
pub struct SheetCell {
    // The cell's type.
    cell_type: CellType,
    // The cell text data.
    text: String,
    // The hook to override a reports cell definition
    layout: Option<CellLayout>,
}
impl SheetCell {
    /// Create a sheet cell with a cell type of [Header](CellType::Header).
    ///
    /// # Arguments
    ///
    /// - `text` is the cell text data.
    ///
    pub fn header(text: impl ToString) -> Self {
        Self { cell_type: CellType::Header, text: text.to_string(), layout: None }
    }
    /// Create a sheet cell with a cell type of [Separator](CellType::Separator).
    ///
    /// # Arguments
    ///
    /// - `text` is the cell separator text.
    ///
    pub fn separator(text: impl ToString) -> Self {
        Self { cell_type: CellType::Separator, text: text.to_string(), layout: None }
    }
    /// Create a sheet cell with a cell type of [Text](CellType::Text).
    ///
    /// # Arguments
    ///
    /// - `text` is the cell text data.
    ///
    pub fn text(text: impl ToString) -> Self {
        Self { cell_type: CellType::Text, text: text.to_string(), layout: None }
    }
    /// Create a sheet cell with a cell type of [Plain](CellType::Plain).
    ///
    /// # Arguments
    ///
    /// - `text` is the cell text data.
    ///
    pub fn plain(text: impl ToString) -> Self {
        Self { cell_type: CellType::Plain, text: text.to_string(), layout: Some(CellLayout::default()) }
    }
    /// A builder method that adds a cell layout to the cell.
    ///
    /// # Arguments
    ///
    /// - `layout` is the cell layout that will be used.
    ///
    pub fn with_layout(mut self, layout: CellLayout) -> Self {
        if self.cell_type != CellType::Plain {
            self.layout = Some(layout);
        }
        self
    }
}

/// The description of a [report sheet](ReportSheet) columns.
///
#[derive(Debug)]
struct SheetLayout {
    /// The cell layouts defined for each column in the report.
    layouts: Vec<CellLayout>,
    /// The default cell layout.
    default_layout: CellLayout,
}
impl SheetLayout {
    /// Get the layout of a specific column.
    ///
    /// # Arguments
    ///
    /// - `index` is the column cell layout to return. The [default](Self::default_layout) cell
    /// layout will be used if the column index is out of bounds.
    ///
    pub fn get(&self, index: usize) -> &CellLayout {
        self.layouts.get(index).unwrap_or_else(|| &self.default_layout)
    }
    /// Get a mutable cell layout for a specific column.
    ///
    /// # Arguments
    ///
    /// - `index` is the column cell layout to return.
    ///
    fn get_mut(&mut self, index: usize) -> Option<&mut CellLayout> {
        self.layouts.get_mut(index)
    }
}

/// The column descriptions and content that comprise a report.
///
#[derive(Debug)]
pub struct ReportSheet {
    /// The report column descriptions.
    layout: SheetLayout,
    /// The report content.
    rows: Vec<Vec<SheetCell>>,
}
impl ReportSheet {
    /// Create a new instance of the report.
    ///
    /// # Arguments
    ///
    /// - `layouts` describe the report column formats.
    ///
    pub fn new(layouts: Vec<CellLayout>) -> Self {
        Self { layout: SheetLayout { layouts, default_layout: CellLayout::default() }, rows: vec![] }
    }
    /// Add a row to the report.
    ///
    /// # Arguments
    ///
    /// - `row` is the textual content.
    ///
    pub fn add_row(&mut self, row: Vec<SheetCell>) {
        for (index, cell) in row.iter().enumerate() {
            if let Some(layout) = self.layout.get_mut(index) {
                adjust_width(layout, cell);
            }
        }
        self.rows.push(row);
    }
    /// Get the number of report columns.
    ///
    pub fn columns(&self) -> usize {
        self.layout.layouts.len()
    }
    /// Get a collection of the cell layouts.
    ///
    pub fn layouts(&self) -> Vec<&CellLayout> {
        self.layout.layouts.iter().collect()
    }
}
impl<'report> IntoIterator for &'report ReportSheet {
    type Item = SheetRow<'report>;
    type IntoIter = ReportSheetIterator<'report>;
    /// Create an iterator that lets you visit each of the report rows.
    ///
    fn into_iter(self) -> Self::IntoIter {
        ReportSheetIterator { report: self, row_index: 0 }
    }
}

/// The report row iterator.
pub struct ReportSheetIterator<'report> {
    /// The report  content.
    report: &'report ReportSheet,
    /// The current row index.
    row_index: usize,
}
impl<'report> Iterator for ReportSheetIterator<'report> {
    type Item = SheetRow<'report>;
    /// Return the next report row.
    fn next(&mut self) -> Option<Self::Item> {
        match self.report.rows.get(self.row_index) {
            None => None,
            Some(cells) => {
                self.row_index += 1;
                Some(SheetRow { layout: &self.report.layout, cells })
            }
        }
    }
}

/// A [reports](ReportSheet) row.
#[derive(Debug)]
pub struct SheetRow<'report> {
    /// The report layout.
    layout: &'report SheetLayout,
    /// The collection of cells that make up the row.
    cells: &'report Vec<SheetCell>,
}
impl<'report> Display for SheetRow<'report> {
    /// Converts row cells into a formatted row.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.into_iter().map(|cell| cell.to_string()).collect::<Vec<String>>().join(" ").trim_end())
    }
}
impl SheetRow<'_> {
    /// Get the number of columns in the row.
    ///
    fn len(&self) -> usize {
        self.cells.len()
    }
    /// Get a column from the row.
    ///
    /// # Arguments
    ///
    /// - `index` is the row column.
    ///
    fn get(&self, index: usize) -> Option<CellData> {
        match self.cells.get(index) {
            None => None,
            Some(cell) => {
                let layout = match &cell.layout {
                    // if the cell does not have a layout, use the sheets
                    None => self.layout.get(index).clone(),
                    Some(cell_layout) => {
                        let sheet_layout = self.layout.get(index);
                        if sheet_layout.alignment.is_none() {
                            // use the cell layout regardless
                            cell_layout.clone()
                        } else {
                            // right now only pay attention to the cells alignment
                            let mut merged_layout = sheet_layout.clone();
                            merged_layout.alignment = cell_layout.alignment;
                            merged_layout.fill = cell_layout.fill;
                            merged_layout
                        }
                    }
                };
                Some(CellData { text: &cell.text, cell_type: cell.cell_type, layout })
            }
        }
    }
}
impl<'row> IntoIterator for &'row SheetRow<'row> {
    type Item = CellData<'row>;
    type IntoIter = SheetRowIterator<'row>;
    /// Return an iterator that visits each column or the row.
    fn into_iter(self) -> Self::IntoIter {
        SheetRowIterator { row: &self, cell_index: 0 }
    }
}

/// An iterator that visits each column of a report row.
pub struct SheetRowIterator<'row> {
    /// The report row that will be visited.
    row: &'row SheetRow<'row>,
    /// The current rows column index.
    cell_index: usize,
}
impl<'row> Iterator for SheetRowIterator<'row> {
    type Item = CellData<'row>;
    /// Get the next column in the report row.
    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.row.get(self.cell_index);
        if cell.is_some() {
            self.cell_index += 1;
        }
        cell
    }
}

/// The metadata for a report row column.
#[derive(Debug)]
pub struct CellData<'row> {
    /// The row column text.
    pub text: &'row str,
    /// The type of row column.
    pub cell_type: CellType,
    /// The row column layout.
    pub layout: CellLayout,
}
impl<'row> Display for CellData<'row> {
    /// Get the column as a formatted string.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.layout.alignment.is_none() {
            write!(f, "{}", self.text)
        } else {
            write!(f, "{}", self.layout.format(self.text))
        }
    }
}

/// Adjust the report column layout.
///
/// # Arguments
///
/// - `layout` is the report column layout.
/// - `cell` is the report row column data.
///
fn adjust_width(layout: &mut CellLayout, cell: &SheetCell) {
    if !layout.alignment.is_none() {
        match &cell.layout {
            None => {
                if !(layout.alignment.is_none() || layout.fixed) {
                    layout.width = cmp::max(layout.width, cell.text.len());
                }
            }
            Some(cell_layout) => {
                if cell_layout.alignment.is_none() {
                    // ignore if there's no alignment
                    ();
                } else if cell_layout.fixed || cell_layout.width > 0 {
                    layout.width = cmp::max(layout.width, cell_layout.width);
                } else {
                    layout.width = cmp::max(layout.width, cell.text.len());
                }
            }
        }
    }
}

/// A helper to create a [report sheets](ReportSheet) [layout](CellLayout).
#[macro_export]
macro_rules! layout {
    (< [$width:expr] +) => {
        $crate::report::CellLayout::new($crate::report::CellAlignment::Left).with_width($width)
    };
    (< [$width:expr]) => {
        $crate::report::CellLayout::new($crate::report::CellAlignment::Left).with_fixed($width)
    };
    (<) => {
        $crate::report::CellLayout::new($crate::report::CellAlignment::Left)
    };
    (^ [$width:expr] +) => {
        $crate::report::CellLayout::new($crate::report::CellAlignment::Center).with_width($width)
    };
    (^ [$width:expr]) => {
        $crate::report::CellLayout::new($crate::report::CellAlignment::Center).with_fixed($width)
    };
    (^) => {
        $crate::report::CellLayout::new($crate::report::CellAlignment::Center)
    };
    (> [$width:expr] +) => {
        $crate::report::CellLayout::new($crate::report::CellAlignment::Right).with_width($width)
    };
    (> [$width:expr]) => {
        $crate::report::CellLayout::new($crate::report::CellAlignment::Right).with_fixed($width)
    };
    (>) => {
        $crate::report::CellLayout::new($crate::report::CellAlignment::Right)
    };
    (* [$width:expr] +) => {
        $crate::report::CellLayout::new($crate::report::CellAlignment::Left).with_width($width).with_fill()
    };
    (* [$width:expr]) => {
        $crate::report::CellLayout::new($crate::report::CellAlignment::Left).with_fixed($width).with_fill()
    };
    (*) => {
        $crate::report::CellLayout::new($crate::report::CellAlignment::Left).with_fill()
    };
    () => {
        $crate::report::CellLayout::new($crate::report::CellAlignment::None).with_fill()
    };
}

/// A helper to create a [report sheet](ReportSheet) row [column](SheetCell) content.
#[macro_export]
macro_rules! text {
        (< $text:expr) => {
            $crate::report::SheetCell::text($text).with_layout(layout!(<))
        };
        (^ $text:expr) => {
            $crate::report::SheetCell::text($text).with_layout(layout!(^))
        };
        (> $text:expr) => {
            $crate::report::SheetCell::text($text).with_layout(layout!(>))
        };
        (+ $text:expr) => {
            $crate::report::SheetCell::text($text).with_layout(layout!(<).with_fill())
        };
        ($text:expr) => {
            $crate::report::SheetCell::text($text)
        };
    }

/// A helper to create a [report sheet](ReportSheet) row header [column](SheetCell).
#[macro_export]
macro_rules! header {
        (< $text:expr) => {
            $crate::report::SheetCell::header($text).with_layout(layout!(<))
        };
        (^ $text:expr) => {
            $crate::report::SheetCell::header($text).with_layout(layout!(^))
        };
        (> $text:expr) => {
            $crate::report::SheetCell::header($text).with_layout(layout!(>))
        };
        (+ $text:expr) => {
            $crate::report::SheetCell::header($text).with_layout(layout!(<).with_fill())
        };
        ($text:expr) => {
            $crate::report::SheetCell::header($text)
        };
    }

/// A helper to create a [report sheet](ReportSheet) separator row.
#[macro_export]
macro_rules! separator {
    (* $separator:expr) => {
        $crate::report::SheetCell::separator($separator).with_layout(layout!(*))
    };
    ($separator:expr) => {
        $crate::report::SheetCell::separator($separator)
    };
}

/// A helper to create a [report sheet](ReportSheet) row plain text [column](SheetCell).
#[macro_export]
macro_rules! plain {
    ($text:expr) => {
        $crate::report::SheetCell::plain($text)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! cell_format {
        ($align:expr, $width:expr, $fixed:expr, $fill:expr) => {
            CellLayout { alignment: $align, width: $width, fixed: $fixed, fill: $fill }
        };
    }

    #[test]
    fn definition() {
        assert_eq!(CellLayout::default(), cell_format!(CellAlignment::None, 0, false, false));
        assert_eq!(CellLayout::new(CellAlignment::Left), cell_format!(CellAlignment::Left, 0, false, false));
        assert_eq!(CellLayout::new(CellAlignment::Center), cell_format!(CellAlignment::Center, 0, false, false));
        assert_eq!(CellLayout::new(CellAlignment::Right), cell_format!(CellAlignment::Right, 0, false, false));
        // bean behaviours
        let mut testcase = CellLayout::new(CellAlignment::Left);
        testcase = testcase.with_fixed(10);
        assert_eq!(testcase, cell_format!(CellAlignment::Left, 10, true, false));
        testcase = testcase.with_width(20);
        assert_eq!(testcase, cell_format!(CellAlignment::Left, 20, false, false));
        testcase = testcase.with_fill();
        assert_eq!(testcase, cell_format!(CellAlignment::Left, 20, false, true));
        let mut testcase = CellLayout::new(CellAlignment::None);
        testcase = testcase.with_fixed(10);
        assert_eq!(testcase, cell_format!(CellAlignment::None, 0, false, false));
        testcase = testcase.with_width(20);
        assert_eq!(testcase, cell_format!(CellAlignment::None, 0, false, false));
        testcase = testcase.with_fill();
        assert_eq!(testcase, cell_format!(CellAlignment::None, 0, false, false));
    }

    #[test]
    fn definition_macros() {
        assert_eq!(layout!(<), cell_format!(CellAlignment::Left, 0, false, false));
        assert_eq!(layout!(< [1]), cell_format!(CellAlignment::Left, 1, true, false));
        assert_eq!(layout!(< [2] +), cell_format!(CellAlignment::Left, 2, false, false));
        assert_eq!(layout!(^), cell_format!(CellAlignment::Center, 0, false, false));
        assert_eq!(layout!(^ [3]), cell_format!(CellAlignment::Center, 3, true, false));
        assert_eq!(layout!(^ [4] +), cell_format!(CellAlignment::Center, 4, false, false));
        assert_eq!(layout!(>), cell_format!(CellAlignment::Right, 0, false, false));
        assert_eq!(layout!(> [5]), cell_format!(CellAlignment::Right, 5, true, false));
        assert_eq!(layout!(> [6] +), cell_format!(CellAlignment::Right, 6, false, false));
        assert_eq!(layout!(*), cell_format!(CellAlignment::Left, 0, false, true));
        assert_eq!(layout!(*[7]), cell_format!(CellAlignment::Left, 7, true, true));
        assert_eq!(layout!(* [8] +), cell_format!(CellAlignment::Left, 8, false, true));
    }

    #[test]
    fn cell() {
        let mut header = SheetCell::header("header");
        assert_eq!(header, SheetCell { cell_type: CellType::Header, text: "header".to_string(), layout: None });
        header = header.with_layout(CellLayout::default());
        assert_eq!(
            header,
            SheetCell { cell_type: CellType::Header, text: "header".to_string(), layout: Some(CellLayout::default()) }
        );
        let mut separator = SheetCell::separator("-");
        assert_eq!(separator, SheetCell { cell_type: CellType::Separator, text: "-".to_string(), layout: None });
        separator = separator.with_layout(CellLayout::default());
        assert_eq!(
            separator,
            SheetCell { cell_type: CellType::Separator, text: "-".to_string(), layout: Some(CellLayout::default()) }
        );
        let mut text = SheetCell::text("text");
        assert_eq!(text, SheetCell { cell_type: CellType::Text, text: "text".to_string(), layout: None });
        text = text.with_layout(CellLayout::default());
        assert_eq!(
            text,
            SheetCell { cell_type: CellType::Text, text: "text".to_string(), layout: Some(CellLayout::default()) }
        );
        let mut plain = SheetCell::plain("plain");
        assert_eq!(
            plain,
            SheetCell {
                cell_type: CellType::Plain,
                text: "plain".to_string(),
                layout: Some(CellLayout::new(CellAlignment::None))
            }
        );
        plain = plain.with_layout(CellLayout::new(CellAlignment::Left));
        assert_eq!(
            plain,
            SheetCell { cell_type: CellType::Plain, text: "plain".to_string(), layout: Some(CellLayout::default()) }
        );
    }

    #[test]
    fn cell_macros() {
        assert_eq!(text!("text"), SheetCell::text("text"));
        assert_eq!(text!(<"text"), SheetCell::text("text").with_layout(CellLayout::new(CellAlignment::Left)));
        assert_eq!(text!(^ "text"), SheetCell::text("text").with_layout(CellLayout::new(CellAlignment::Center)));
        assert_eq!(text!(> "text"), SheetCell::text("text").with_layout(CellLayout::new(CellAlignment::Right)));
        assert_eq!(text!(+ "-"), SheetCell::text("-").with_layout(CellLayout::new(CellAlignment::Left).with_fill()));
        assert_eq!(header!("header"), SheetCell::header("header"));
        assert_eq!(header!(< "header"), SheetCell::header("header").with_layout(CellLayout::new(CellAlignment::Left)));
        assert_eq!(header!(^ "header"), SheetCell::header("header").with_layout(CellLayout::new(CellAlignment::Center)));
        assert_eq!(header!(> "header"), SheetCell::header("header").with_layout(CellLayout::new(CellAlignment::Right)));
        assert_eq!(header!(+ "-"), SheetCell::header("-").with_layout(CellLayout::new(CellAlignment::Left).with_fill()));
        assert_eq!(plain!("plain"), SheetCell::plain("plain"));
        assert_eq!(separator!("separator"), SheetCell::separator("separator"));
        assert_eq!(
            separator!(*"separator"),
            SheetCell::separator("separator").with_layout(CellLayout::new(CellAlignment::Left).with_fill())
        );
    }

    // #[test]
    // fn text() {
    //     let text = "The quick brown fox";
    //     assert_eq!(cell_text(text, CellAlignment::Left, 19), "The quick brown fox");
    //     assert_eq!(cell_text(text, CellAlignment::Left, 10), "The quick ");
    //     assert_eq!(cell_text(text, CellAlignment::Right, 10), " brown fox");
    //     assert_eq!(cell_text(text, CellAlignment::Center, 10), "quick brow");
    // }

    #[test]
    fn format() {
        assert_eq!(CellLayout::new(CellAlignment::Left).with_width(5).format("left"), "left ");
        assert_eq!(CellLayout::new(CellAlignment::Center).with_width(8).format("center"), " center ");
        assert_eq!(CellLayout::new(CellAlignment::Right).with_width(6).format("right"), " right");
        assert_eq!(CellLayout::new(CellAlignment::Left).with_width(6).with_fill().format("-"), "------");
    }

    #[test]
    fn data_format() {
        let test_string = "testcase";
        assert_eq!(CellData { text: "left", cell_type: CellType::Text, layout: layout!(< [5] +) }.to_string(), "left ");
        assert_eq!(
            CellData { text: "center", cell_type: CellType::Text, layout: layout!(^ [8]) }.to_string(),
            " center "
        );
        assert_eq!(CellData { text: "right", cell_type: CellType::Text, layout: layout!(> [6]) }.to_string(), " right");
        assert_eq!(
            CellData { text: "plain cell", cell_type: CellType::Text, layout: layout!() }.to_string(),
            "plain cell"
        );
    }

    #[test]
    fn width() {
        // ignore if the cell layout is none
        let mut testcase = CellLayout::new(CellAlignment::None);
        adjust_width(&mut testcase, &SheetCell::text("doesn't matter"));
        assert_eq!(testcase, CellLayout::new(CellAlignment::None));
        // check the auto sizing layout
        let mut testcase = CellLayout::new(CellAlignment::Left).with_width(5);
        adjust_width(&mut testcase, &SheetCell::text("less"));
        assert_eq!(testcase, CellLayout::new(CellAlignment::Left).with_width(5));
        adjust_width(&mut testcase, &SheetCell::text("bigger"));
        assert_eq!(testcase, CellLayout::new(CellAlignment::Left).with_width(6));
        // check the fixed size layout
        let mut testcase = CellLayout::new(CellAlignment::Left).with_fixed(5);
        adjust_width(&mut testcase, &SheetCell::text("less"));
        assert_eq!(testcase, CellLayout::new(CellAlignment::Left).with_fixed(5));
        adjust_width(&mut testcase, &SheetCell::text("bigger"));
        assert_eq!(testcase, CellLayout::new(CellAlignment::Left).with_fixed(5));
        // make sure that a plain cell doesn't clobber the sheet layout width
        let mut testcase = CellLayout::new(CellAlignment::Left).with_width(5);
        adjust_width(&mut testcase, &SheetCell::plain("bigger"));
        assert_eq!(testcase, CellLayout::new(CellAlignment::Left).with_width(5));
        // make sure the layout auto sizes for cells with layouts
        let mut testcase = CellLayout::new(CellAlignment::Left).with_width(5);
        adjust_width(&mut testcase, &SheetCell::text("bigger"));
        assert_eq!(testcase, CellLayout::new(CellAlignment::Left).with_width(6));
        adjust_width(
            &mut testcase,
            &SheetCell::text("fixed").with_layout(CellLayout::new(CellAlignment::Left).with_fixed(10)),
        );
        assert_eq!(testcase, CellLayout::new(CellAlignment::Left).with_width(10));
        adjust_width(
            &mut testcase,
            &SheetCell::text("width").with_layout(CellLayout::new(CellAlignment::Left).with_fixed(15)),
        );
        assert_eq!(testcase, CellLayout::new(CellAlignment::Left).with_width(15));
    }

    #[test]
    fn row() {
        let layout = SheetLayout {
            layouts: vec![layout!(< [1]), layout!(^ [5]), layout!(^ [8]), layout!(> [6])],
            default_layout: Default::default(),
        };
        let cells = vec![
            plain!("override"),
            // use the cell layout not the sheets
            text!(< "left"),
            text!("center"),
            text!("right"),
            // there should not be a layout for this cell
            text!("plain data"),
        ];
        let row = SheetRow { layout: &layout, cells: &cells };
        let mut testcase = row.into_iter();
        assert_eq!(testcase.next().unwrap().to_string(), "override");
        assert_eq!(testcase.next().unwrap().to_string(), "left ");
        assert_eq!(testcase.next().unwrap().to_string(), " center ");
        assert_eq!(testcase.next().unwrap().to_string(), " right");
        assert_eq!(testcase.next().unwrap().to_string(), "plain data");
        assert!(testcase.next().is_none());
    }

    #[test]
    fn report() {
        let mut report = ReportSheet::new(vec![layout!(>), layout!(^ [4]+), layout!(< [10])]);
        report.add_row(vec![header!(^"h1"), header!(^"hdr2"), header!(^"hdr3")]);
        assert_eq!(report.layout.layouts[0].width, 2);
        assert_eq!(report.layout.layouts[1].width, 4);
        assert_eq!(report.layout.layouts[2].width, 10);
        report.add_row(vec![text!("10."), text!("R1/C2"), text!("R1/C3")]);
        assert_eq!(report.layout.layouts[0].width, 3);
        assert_eq!(report.layout.layouts[1].width, 5);
        assert_eq!(report.layout.layouts[2].width, 10);
        report.add_row(vec![text!("100."), plain!("R2/C2 with plain text"), text!("R2 C3 with long text")]);
        assert_eq!(report.layout.layouts[0].width, 4);
        assert_eq!(report.layout.layouts[1].width, 5);
        assert_eq!(report.layout.layouts[2].width, 10);
        let mut iter = report.into_iter();
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert!(iter.next().is_none())
    }
}
