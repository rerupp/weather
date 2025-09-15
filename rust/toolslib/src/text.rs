//! # A text based report generator
//!
//! The intent of this module is to provide a common text based reporting engine.
//! There was so much commonality between the various cli reporting commands it
//! seemed reasonable to build a common reporting engine.
//!
//! The components allow text to be placed into report columns, abstracting how the text is
//! really generated. At some point I would think defining a set of macros to help genearate
//! the output will be in order.

// use std::{fmt::{self, Alignment}, fs, io, iter::Iterator, path::PathBuf, result};
use std::{fmt, fs, io, iter::Iterator, path::PathBuf, result, string::ToString};

/// The text module result.
type Result<T> = result::Result<T, Error>;

/// The text Error that can be captured outside the module.
///
/// Currently it contains only a String but can be extended to an enum later on.
#[derive(Debug)]
pub struct Error(String);
/// Include the `ToString` trait for the [`Error`].
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
/// Create a text error from a String.
impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::from(error.as_str())
    }
}
/// Create a text error from a str slice.
impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Error(format!("::text: {error}"))
    }
}
/// Create a text error from an `io::Error`.
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::from(error.to_string())
    }
}

/// Gets a `io::Write` writer for either a file or `stdout`.
///
/// # Arguments
///
/// * `file_option` - if `None` then `stdout` will be used otherwise the file path will be opened.
/// * `append` - if writing to a file, append output if `true` otherwise truncate existing file
///   contents.
pub fn get_writer(file_option: &Option<PathBuf>, append: bool) -> Result<Box<dyn io::Write>> {
    if file_option.is_none() {
        Ok(Box::new(io::stdout()))
    } else {
        let file_path = file_option.as_ref().unwrap();
        let mut open_options = fs::OpenOptions::new();
        if append {
            open_options.append(true);
        } else {
            open_options.write(true).truncate(true).create(true);
        }
        match open_options.open(file_path.as_path().display().to_string()) {
            Ok(writer) => Ok(Box::new(std::io::BufWriter::new(writer))),
            Err(error) => {
                let errmsg = format!("Error opening {}: {error}", file_path.as_path().display().to_string());
                log::error!("{errmsg}");
                Err(Error::from(errmsg))
            }
        }
    }
}

/// Writes a collection of strings.
///
/// # Arguments
///
/// * `writer` is where text will be written.
/// * `string_iter` is the source of what will be written.
pub fn write_strings<T: Iterator<Item = String>>(writer: &mut dyn io::Write, string_iter: T) -> Result<()> {
    for string in string_iter {
        writeln!(writer, "{}", string.as_str())?;
    }
    writer.flush()?;
    Ok(())
}

/// Indicate the alignment of a data cell.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Alignment {
    /// Text will be left aligned.
    Left,
    /// Text will be center aligned.
    Center,
    /// Text will be right aligned.
    Right,
    /// Text will be repeated to fill the column.
    Span,
}

/// The description of a column in a report
#[derive(Debug, PartialEq)]
pub struct ReportColumn {
    /// The default alignment of text for a report column.
    alignment: Alignment,
    /// The character width of a column.
    width: usize,
    /// When `true` the width of the column is fixed in length.
    fixed_width: bool,
}
impl ReportColumn {
    /// Creates a new instance of the report column.
    ///
    /// # Arguments
    ///
    /// * `alignment` is the default alignment of text for the report column.
    /// * `width` is the initial width of the report column.
    /// * `fixed_width` indicates whether or not the report column is fixed width.
    pub fn new(alignment: Alignment, width: usize, fixed_width: bool) -> Self {
        Self { alignment, width, fixed_width }
    }
    /// Identifies if column text should be added to the report as is.
    pub fn ignore_alignment(&self) -> bool {
        self.fixed_width && self.width == 0
    }
}

/// The column content of a [`Report`] row.
#[derive(Debug, PartialEq)]
pub struct ReportData {
    /// The columns textual data.
    data: String,
    /// If provided, the alignment will override the default column alignment.
    alignment: Option<Alignment>,
    /// If true the data will be output as is.
    as_is: bool,
}
impl ReportData {
    /// Creates an instance of the report data.
    ///
    /// # Arguments
    ///
    /// * `value` is the data that will be used for the [`Report`] column.
    /// * `alignment` is the desired alignment of the [`Report`] column.
    pub fn new<T: fmt::Display>(value: T, alignment: Option<Alignment>) -> Self {
        Self { data: value.to_string(), alignment, as_is: false }
    }
    /// Creates an instance of the report data with `alignment` set to None and `as_is` set to `true`.
    ///
    /// # Arguments
    ///
    /// * `value` is the data that will be used for the [`Report`] column.
    pub fn as_is<T: fmt::Display>(value: T) -> Self {
        Self { data: value.to_string(), alignment: None, as_is: true }
    }
    /// Formats the report data using the report column defintion.
    ///
    /// # Arguments
    ///
    /// * `report_column` is the associated report column definition.
    pub fn fmt(&self, report_column: &ReportColumn) -> String {
        let width = if self.as_is {
            // irregardless the column format, use the data length
            self.data.len()
        } else if report_column.ignore_alignment() {
            // use the data length if the column is configured as is
            self.data.len()
        } else {
            report_column.width
        };
        let alignment = self.alignment.as_ref().unwrap_or_else(|| &report_column.alignment);
        let data_len = self.data.len();
        let data = if data_len <= width {
            &self.data[..]
        } else {
            match alignment {
                Alignment::Left => {
                    // truncate the rhs
                    &self.data[0..width]
                }
                Alignment::Center | &Alignment::Span => {
                    // truncate the lhs and rhs
                    let offset = (data_len - width) / 2;
                    &self.data[offset..offset + width]
                }
                Alignment::Right => {
                    // truncate the lhs
                    let offset = data_len - width;
                    &self.data[offset..]
                }
            }
        };
        match alignment {
            Alignment::Left => format!("{data:<width$}"),
            Alignment::Center => format!("{data:^width$}"),
            Alignment::Right => format!("{data:>width$}"),
            Alignment::Span => {
                if data_len == width {
                    data.to_string()
                } else {
                    let repeat = (width / data_len) + 1;
                    data.repeat(repeat)[0..width].to_string()
                }
            }
        }
    }
}

/// The type of row that has been added to a [`Report`].
#[derive(Debug, PartialEq)]
pub enum ReportRow {
    /// This variant is a header and holds the collection of [`ReportData`] used to generate the header
    /// rows text.
    Header(Vec<ReportData>),
    /// This variant is a separator and holds the data used to generate the rows text.
    Separator(String),
    /// This variant is content and holds the collection of [`ReportData`] used to generate the rows text.
    Text(Vec<ReportData>),
}
impl ReportRow {
    /// Generates a row of text based on the collectioni of report columns.
    ///
    /// For each `ReportRow` variant:
    ///
    /// * [`Header`](ReportRow::Header) delegates row creation to the [`format_header`] function.
    /// * [`Separator`](ReportRow::Separator) delegates row creation to the [`format_separator`] function.
    /// * [`Text`](ReportRow::Text) delegates row creation to the [`format_text`] function.
    ///
    /// # Arguments
    ///
    /// * `report_columns` contains the report column descriptions.
    fn generate(&self, report_columns: &Vec<ReportColumn>) -> String {
        match self {
            ReportRow::Header(headers) => format_header(report_columns, headers),
            ReportRow::Separator(separator) => format_separator(report_columns, separator),
            ReportRow::Text(columns) => format_text(report_columns, columns),
        }
    }
}

/// A container of report row column descriptions and content.
#[derive(Debug, PartialEq)]
pub struct Report {
    /// The collection of report row column descriptions.
    report_columns: Vec<ReportColumn>,
    /// The collection of report rows.
    report_rows: Vec<ReportRow>,
}
impl From<Vec<ReportColumn>> for Report {
    fn from(rc: Vec<ReportColumn>) -> Self {
        Self { report_columns: rc, report_rows: vec![] }
    }
}
impl Report {
    /// Adds a header row to the report.
    ///
    /// # Arguments
    ///
    /// * `header_row` is the collection report data that comprise the header row.
    pub fn header(&mut self, row: Vec<ReportData>) -> &mut Self {
        self.adjust_column_widths(&row);
        self.report_rows.push(ReportRow::Header(row));
        self
    }
    /// Adds a separator row to the report.
    ///
    /// # Arguments
    ///
    /// * `separator` is the string used to create the separator row.
    pub fn separator(&mut self, separator: &str) -> &mut Self {
        self.report_rows.push(ReportRow::Separator(separator.to_string()));
        self
    }
    /// Adds a text row to the report.
    ///
    /// # Arguments
    ///
    /// * `text_row` is the collection report data that comprise the rows content.
    pub fn text(&mut self, row: Vec<ReportData>) -> &mut Self {
        self.adjust_column_widths(&row);
        self.report_rows.push(ReportRow::Text(row));
        self
    }
    /// An internal function that adjusts the width of each report column description.
    ///
    /// A columns width will not be adjusted if:
    ///
    /// * the report data has been set [as is](ReportData::as_is).
    /// * the column description has been set to [fixed width](ReportColumn::fixed_width).
    fn adjust_column_widths(&mut self, report_data: &Vec<ReportData>) {
        for i in 0..std::cmp::min(self.report_columns.len(), report_data.len()) {
            let data = &report_data[i];
            if !data.as_is {
                let column_format = self.report_columns.get_mut(i).unwrap();
                if !column_format.fixed_width {
                    column_format.width = std::cmp::max(column_format.width, data.data.len());
                }
            }
        }
    }
}

/// Allows the report to be converted to an iterator that returns row of the report.
impl<'r> IntoIterator for &'r Report {
    /// The iterator implementation for a report.
    type IntoIter = ReportIterator<'r>;
    /// Each row of the report is returned as a string.
    type Item = String;
    /// Creates the report builder iterator.
    fn into_iter(self) -> Self::IntoIter {
        ReportIterator { report: self, row_index: 0 }
    }
}

/// The report row iterator.
pub struct ReportIterator<'r> {
    /// A reference to the report container.
    report: &'r Report,
    /// The report row returned when `next` is called.
    row_index: usize,
}

/// The report row iterator used to return the rows of a report.
impl<'r> Iterator for ReportIterator<'r> {
    type Item = String;
    /// Creates a line of text output for the report.
    fn next(&mut self) -> Option<Self::Item> {
        match self.report.report_rows.get(self.row_index) {
            Some(row) => {
                self.row_index += 1;
                Some(row.generate(&self.report.report_columns).trim_end().to_string())
            }
            None => None,
        }
    }
}

/// Creates a line of header text using the collection of [`ReportColumn`] and collection of [`ReportData`].
///
/// See [`format_text`] for details about how the header text will be formatted.
///
/// # Arguments
///
/// * `cols` is the collection of column definitions describing the report header row.
/// * `headers` is the collection of header text data used to populate the report row.
fn format_header(cols: &Vec<ReportColumn>, headers: &Vec<ReportData>) -> String {
    format_text(cols, headers)
}

/// Create a line of text with each report column containing the separator.
///
/// If the separator string length is less than the report column width, the
/// separator string will be repeated until if fills the report columns width.
/// If the column width is 0, the separator will not be added to the line of text.
///
/// # Arguments
///
/// * `cols` is the collection of report column definitions.
/// * `separator` is the separator string that will fill each of the report columns.
fn format_separator(cols: &Vec<ReportColumn>, separator: &str) -> String {
    let mut row_text = String::from("");
    let separator_len = separator.len();
    cols.iter().for_each(|report_column| {
        if !row_text.is_empty() {
            row_text.push(' ');
        }
        if report_column.width == 0 {
            ();
        } else if separator_len < 2 {
            row_text.push_str(&separator.repeat(report_column.width));
        } else if separator_len == report_column.width {
            row_text.push_str(separator);
        } else if separator_len > report_column.width {
            row_text.push_str(&separator[0..report_column.width]);
        } else {
            let repeat_count = 1 + (((report_column.width - separator_len) as f64 / 2.0) + 0.5) as usize;
            let separator_text = separator.repeat(repeat_count);
            row_text.push_str(&separator_text[0..report_column.width]);
        }
    });
    row_text
}

/// Creates a line of text using the collection of [`ReportColumn`] and collection of [`ReportData`].
///
/// The collection of report column definitions can be larger than the collection of report data.
/// if there is more report data than report column defintions, the report data will be output as is
/// separated by a space.
///
/// # Arguments
///
/// * `cols` is the collection of column definitions describing the report row.
/// * `row` is the collection of text data used to populate the report row.
fn format_text(cols: &Vec<ReportColumn>, row: &Vec<ReportData>) -> String {
    let col_formats_len = cols.len();
    let text_columns_len = row.len();
    let mut row_text = String::new();
    for i in 0..std::cmp::min(col_formats_len, text_columns_len) {
        if !row_text.is_empty() {
            row_text.push(' ');
        }
        row_text.push_str(&row[i].fmt(&cols[i]));
    }
    if col_formats_len < text_columns_len {
        const AS_IS: ReportColumn = ReportColumn { alignment: Alignment::Left, width: 0, fixed_width: true };
        for i in col_formats_len..text_columns_len {
            row_text.push(' ');
            row_text.push_str(&row[i].fmt(&AS_IS));
        }
    }
    row_text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rptcols, rptrow};
    #[test]
    fn error() {
        assert_eq!(Error::from("foo").0, "::text: foo");
        assert_eq!(Error::from(String::from("bar")).0, String::from("::text: bar"));
        assert_eq!(format!("{}", Error::from("raboof")), format!("::text: raboof"));
    }
    #[test]
    fn write_strings_fn() {
        let source: Vec<String> = ["one", "two", "three"].iter().map(|&s| s.to_string()).collect();
        let mut buffer = Vec::new();
        write_strings(&mut buffer, source.clone().into_iter()).unwrap();
        let testcase = String::from_utf8(buffer).unwrap();
        let mut lines = testcase.lines();
        assert_eq!(lines.next(), Some(source[0].as_str()));
        assert_eq!(lines.next(), Some(source[1].as_str()));
        assert_eq!(lines.next(), Some(source[2].as_str()));
        assert_eq!(lines.next(), None);
    }
    #[test]
    fn report_column() {
        let testcase = ReportColumn::new(Alignment::Left, 0, false);
        assert_eq!(testcase.alignment, Alignment::Left);
        assert_eq!(testcase.width, 0);
        assert_eq!(testcase.fixed_width, false);
        let testcase = ReportColumn::new(Alignment::Center, 10, false);
        assert_eq!(testcase.alignment, Alignment::Center);
        assert_eq!(testcase.width, 10);
        assert_eq!(testcase.fixed_width, false);
        let testcase = ReportColumn::new(Alignment::Right, 20, true);
        assert_eq!(testcase.alignment, Alignment::Right);
        assert_eq!(testcase.width, 20);
        assert_eq!(testcase.fixed_width, true);
        assert!(ReportColumn::new(Alignment::Left, 0, true).ignore_alignment())
    }
    #[test]
    fn report_data() {
        let testcase = ReportData::new(12345, Some(Alignment::Center));
        assert_eq!(testcase.data, "12345");
        assert_eq!(testcase.alignment, Some(Alignment::Center));
        let testcase = ReportData::new("abcd", None);
        assert_eq!(testcase.data, "abcd");
        assert_eq!(testcase.alignment, None);
        let column_format = ReportColumn::new(Alignment::Left, 5, false);
        assert_eq!(ReportData::new("abc", None).fmt(&column_format), "abc  ");
        assert_eq!(ReportData::new("abc", Some(Alignment::Center)).fmt(&column_format), " abc ");
        assert_eq!(ReportData::new("abc", Some(Alignment::Right)).fmt(&column_format), "  abc");
        let column_format = ReportColumn::new(Alignment::Left, 0, true);
        assert_eq!(ReportData::new("something", Some(Alignment::Left)).fmt(&column_format), "something");
        assert_eq!(ReportData::new("shorter", Some(Alignment::Left)).fmt(&column_format), "shorter");
        assert_eq!(ReportData::new("one longer", Some(Alignment::Left)).fmt(&column_format), "one longer");
        assert_eq!(ReportData::as_is("something").fmt(&column_format), "something");
        assert_eq!(ReportData::as_is("shorter").fmt(&column_format), "shorter");
        assert_eq!(ReportData::as_is("one longer").fmt(&column_format), "one longer");
        let testcase = ReportData::new("abcde", None);
        assert_eq!(testcase.fmt(&ReportColumn::new(Alignment::Left, 3, false)), "abc");
        assert_eq!(testcase.fmt(&ReportColumn::new(Alignment::Center, 3, false)), "bcd");
        assert_eq!(testcase.fmt(&ReportColumn::new(Alignment::Center, 2, false)), "bc");
        assert_eq!(testcase.fmt(&ReportColumn::new(Alignment::Center, 4, false)), "abcd");
        assert_eq!(testcase.fmt(&ReportColumn::new(Alignment::Right, 3, true)), "cde");
        assert_eq!(testcase.fmt(&ReportColumn::new(Alignment::Span, 3, true)), "bcd");
        assert_eq!(testcase.fmt(&ReportColumn::new(Alignment::Span, 6, true)), "abcdea");
        assert_eq!(testcase.fmt(&ReportColumn::new(Alignment::Span, 10, true)), "abcdeabcde");
    }
    #[test]
    fn format_text_fn() {
        let column_formats = rptcols!(<+(10), ^+(5), >+(10));
        let testcase = format_text(&column_formats, &rptrow!("hello", "-", "there"));
        assert_eq!(testcase, format!("{} {} {}", "hello     ", "  -  ", "     there"));
        let testcase = format_text(&column_formats, &rptrow!("another", "try"));
        assert_eq!(testcase, format!("{} {}", "another   ", " try "));
        let testcase = format_text(&column_formats, &rptrow!("too", "many", "text", "columns"));
        assert_eq!(testcase, format!("{} {} {} {}", "too       ", "many ", "      text", "columns"));
    }
    #[test]
    fn format_text_fixed_width() {
        let column_formats = rptcols!(=, ^+(5), =);
        let testcase = format_text(&column_formats, &rptrow!("hello", "-", "there"));
        assert_eq!(testcase, format!("{} {} {}", "hello", "  -  ", "there"));
        let testcase = format_text(&column_formats, &rptrow!("lets", "tryit", "one more time"));
        assert_eq!(testcase, format!("{} {} {}", "lets", "tryit", "one more time"));
    }
    #[test]
    fn format_separator_fn() {
        let column_formats = rptcols!(<+(1), <+(2), <+(5));
        let testcase = format_separator(&column_formats, "");
        assert_eq!(testcase, String::default());
        let testcase = format_separator(&column_formats, "-");
        assert_eq!(testcase, String::from("- -- -----"));
        let testcase = format_separator(&column_formats, "+-");
        assert_eq!(testcase, String::from("+ +- +-+-+"));
        let testcase = format_separator(&column_formats, "+-=");
        assert_eq!(testcase, String::from("+ +- +-=+-"));
    }
    #[test]
    fn report() {
        let mut report = Report::from(rptcols!(<, ^, >));
        report.header(rptrow!(^ "#", "TestCase", ^ "Value"));
        report.separator("-");
        report.text(rptrow!(1, "TC1", 45.6));
        report.text(rptrow!("Two", "TC2", (4 + 5)));
        let mut testcase = report.into_iter();
        assert_eq!(testcase.next().unwrap(), format!("{} {} {}", " # ", "TestCase", "Value"));
        assert_eq!(testcase.next().unwrap(), "--- -------- -----");
        assert_eq!(testcase.next().unwrap(), format!("{} {} {}", "1  ", "  TC1   ", " 45.6"));
        assert_eq!(testcase.next().unwrap(), format!("{} {} {}", "Two", "  TC2   ", "    9"));
        assert_eq!(testcase.next(), None);
    }
    #[test]
    fn report_fixed_width() {
        let mut report = Report::from(rptcols!(<=(2), <, >));
        report.text(rptrow!(= "Header1"));
        report.text(rptrow!("", = "Some long text"));
        report.text(rptrow!("Hdr2"));
        report.text(rptrow!("", "Short", ="text"));
        report.text(rptrow!("", "Shorter", ="more text"));
        let mut testcase = report.into_iter();
        assert_eq!(testcase.next().unwrap(), "Header1");
        assert_eq!(testcase.next().unwrap(), "   Some long text");
        assert_eq!(testcase.next().unwrap(), "Hd");
        assert_eq!(testcase.next().unwrap(), "   Short   text");
        assert_eq!(testcase.next().unwrap(), "   Shorter more text");
        assert_eq!(testcase.next(), None);

        let mut report = Report::from(rptcols!(=, =));
        report.header(rptrow!(^ "Left", ^ "Right"));
        report.separator("=");
        report.text(rptrow!("First", "row"));
        report.text(rptrow!("Second", "line"));
        let mut testcase = report.into_iter();
        assert_eq!(testcase.next().unwrap(), "Left Right");
        assert_eq!(testcase.next().unwrap(), "");
        assert_eq!(testcase.next().unwrap(), "First row");
        assert_eq!(testcase.next().unwrap(), "Second line");
        assert_eq!(testcase.next(), None);
    }
}

mod macros {
    //! A collection of helper macros that facilitate the creation of a [Report](super::Report).
    //!
    //! Creating a collection of [ReportColumn](super::ReportColumn) and [ReportData](super::ReportData)
	//! can be verbose. The [rptcols](crate::rptcols) and [rptrow](crate::rptrow) macros
    //! ease this verbosity providing simple markup to generate the respective
    //! collections.

    /// Creates and instance of [ReportData](struct@super::ReportData) that can
    /// be added to a [Report](struct@super::Report) row.
    ///
    /// Simple markup facilitates the creation of `ReportData`. The following example shows
    /// the markup and resulting `ReportData` collection.
    ///
    /// ```
    /// # use toolslib::text::{Alignment, ReportData};
    /// use toolslib::rptdata;
    /// assert_eq!(rptdata!(_), ReportData::new("", None));
    /// assert_eq!(rptdata!(+ "spanned"), ReportData::new("spanned", Some(Alignment::Span)));
    /// assert_eq!(rptdata!("data"), ReportData::new("data", None));
    /// assert_eq!(rptdata!(< "left"), ReportData::new("left", Some(Alignment::Left)));
    /// assert_eq!(rptdata!(^ "center"), ReportData::new("center", Some(Alignment::Center)));
    /// assert_eq!(rptdata!(> "right"), ReportData::new("right", Some(Alignment::Right)));
    /// assert_eq!(rptdata!(= "as is"), ReportData::as_is("as is"));
    /// ```
    #[macro_export]
    macro_rules! rptdata {
        // Creates an empty report data column.
        (_) => {
            $crate::text::ReportData::new("", None)
        };
        // Create a spanned report data column.
        (+ $data:expr) => {
            $crate::text::ReportData::new($data, Some($crate::text::Alignment::Span))
        };
        // Create report data that will output as is.
        (= $data:expr) => {
            $crate::text::ReportData::as_is($data)
        };
        // Create left justified report data overriding the report column alignment.
        (< $data:expr) => {
            $crate::text::ReportData::new($data, Some($crate::text::Alignment::Left))
        };
        // Create center justified report data overriding the report column alignment.
        (^ $data:expr) => {
            $crate::text::ReportData::new($data, Some($crate::text::Alignment::Center))
        };
        // Create right justified report data overriding the report column alignment.
        (> $data:expr) => {
            $crate::text::ReportData::new($data, Some($crate::text::Alignment::Right))
        };
        // Create report data that uses the report column alignment.
        ($data:expr) => {
            $crate::text::ReportData::new($data, None)
        };
    }

    /// Generates a collection of [`ReportData`](struct@super::ReportData) that can
    /// be added to a [`Report`](struct@super::Report).
    ///
    /// Simple markup facilitates the creation of `ReportData`. The result of calling this macro is a `Vec<ReportData>`.
    /// The following example shows the markup and resulting `ReportData` collection.
    ///
    /// ```
    /// # use toolslib::text::{Alignment, ReportData};
    /// use toolslib::rptrow;
    /// assert_eq!(
    ///     rptrow!(_, "This", < "is", ^ "a row of", > "report", = "data"),
    ///     vec![
    ///         ReportData::new("", None),
    ///         ReportData::new("This", None),
    ///         ReportData::new("is", Some(Alignment::Left)),
    ///         ReportData::new("a row of", Some(Alignment::Center)),
    ///         ReportData::new("report", Some(Alignment::Right)),
    ///         ReportData::as_is("data"),
    ///     ]
    /// );
    /// ```
    #[macro_export]
    macro_rules! rptrow {
            // the terminal rule that generates a collection of data cells
            (@rd () -> [$($data_cells:tt)*]) => {
                ::std::vec!($($data_cells)*)
            };
            // the left aligned, comma delimited, markup overrides the columns alignment
            (@rd ( _, $($data_markups:tt)*) -> [$($data_cells:tt)*]) => {
                rptrow!(@rd ($($data_markups)*) -> [
                    $($data_cells)*
                    $crate::text::ReportData::new("", None),
                ])
            };
            // the span, comma delimited, markup overrides the columns alignment
            (@rd ( + $data:expr, $($data_markups:tt)*) -> [$($data_cells:tt)*]) => {
                rptrow!(@rd ($($data_markups)*) -> [
                    $($data_cells)*
                    $crate::text::ReportData::new($data, Some($crate::text::Alignment::Span)),
                ])
            };
            // the span markup overrides the columns alignment, it ends markup parsing
            (@rd ( + $data:expr ) -> [$($data_cells:tt)*]) => {
                rptrow!(@rd () -> [
                    $($data_cells)*
                    $crate::text::ReportData::new($data, Some($crate::text::Alignment::Span)),
                ])
            };
            // the left aligned, comma delimited, markup overrides the columns alignment
            (@rd ( = $data:expr, $($data_markups:tt)*) -> [$($data_cells:tt)*]) => {
                rptrow!(@rd ($($data_markups)*) -> [
                    $($data_cells)*
                    $crate::text::ReportData::as_is($data),
                ])
            };
            // the left aligned markup overrides the columns alignment, it ends markup parsing
            (@rd ( = $data:expr ) -> [$($data_cells:tt)*]) => {
                rptrow!(@rd () -> [
                    $($data_cells)*
                    $crate::text::ReportData::as_is($data),
                ])
            };
            // the left aligned, comma delimited, markup overrides the columns alignment
            (@rd ( < $data:expr, $($data_markups:tt)*) -> [$($data_cells:tt)*]) => {
                rptrow!(@rd ($($data_markups)*) -> [
                    $($data_cells)*
                    $crate::text::ReportData::new($data, Some($crate::text::Alignment::Left)),
                ])
            };
            // the left aligned markup overrides the columns alignment, it ends markup parsing
            (@rd ( < $data:expr ) -> [$($data_cells:tt)*]) => {
                rptrow!(@rd () -> [
                    $($data_cells)*
                    $crate::text::ReportData::new($data, Some($crate::text::Alignment::Left)),
                ])
            };
            // the center aligned, comma delimited, markup overrides the columns alignment
            (@rd ( ^ $data:expr, $($data_markups:tt)*) -> [$($data_cells:tt)*]) => {
                rptrow!(@rd ($($data_markups)*) -> [
                    $($data_cells)*
                    $crate::text::ReportData::new($data, Some($crate::text::Alignment::Center)),
                ])
            };
            // the center aligned markup overrides the columns alignment, it ends markup parsing
            (@rd ( ^ $data:expr ) -> [$($data_cells:tt)*]) => {
                rptrow!(@rd () -> [
                    $($data_cells)*
                    $crate::text::ReportData::new($data, Some($crate::text::Alignment::Center)),
                ])
            };
            // the right aligned, comma delimited, markup overrides the columns alignment
            (@rd ( > $data:expr, $($data_markups:tt)*) -> [$($data_cells:tt)*]) => {
                rptrow!(@rd ($($data_markups)*) -> [
                    $($data_cells)*
                    $crate::text::ReportData::new($data, Some($crate::text::Alignment::Right)),
                ])
            };
            // the right aligned markup overrides the columns alignment, it ends markup parsing
            (@rd ( > $data:expr ) -> [$($data_cells:tt)*]) => {
                rptrow!(@rd () -> [
                    $($data_cells)*
                    $crate::text::ReportData::new($data, Some($crate::text::Alignment::Right)),
                ])
            };
            // the comma delimited markup uses the columns alignent
            (@rd ($data:expr, $($data_markups:tt)*) -> [$($data_cells:tt)*]) => {
                rptrow!(@rd ($($data_markups)*) -> [
                    $($data_cells)*
                    $crate::text::ReportData::new($data, None),
                ])
            };
            // use the columns alignment, it ends markup parsing
            (@rd ($data:expr) -> [$($data_cells:tt)*]) => {
                rptrow!(@rd () -> [
                    $($data_cells)*
                    $crate::text::ReportData::new($data, None),
                ])
            };
            // the entry point sets up the tt muncher that parses the data cell markup
            ($($data_markups:tt)*) => {
                rptrow!(@rd ($($data_markups)*) -> [])
            };
        }

    /// This macro generates a collection of [`ReportColumn`](struct@super::ReportColumn) that
    /// describes rows of a [`Report`](struct@super::Report).
    ///
    /// The result of calling this macro is a `Vec<ReportColumn>`. It facilitates the creation
    /// of describing the rows of a `Report`.
    ///
    /// ```
    /// # use toolslib::text::{Alignment, ReportColumn};
    /// use toolslib::rptcols;
    /// assert_eq!(rptcols!(<, <+(1), <=(2), ^, ^+(3), ^=(4), >, >+(5), >=(6), =),
    ///     vec![
    ///         ReportColumn::new(Alignment::Left, 0, false),
    ///         ReportColumn::new(Alignment::Left, 1, false),
    ///         ReportColumn::new(Alignment::Left, 2, true),
    ///         ReportColumn::new(Alignment::Center, 0, false),
    ///         ReportColumn::new(Alignment::Center, 3, false),
    ///         ReportColumn::new(Alignment::Center, 4, true),
    ///         ReportColumn::new(Alignment::Right, 0, false),
    ///         ReportColumn::new(Alignment::Right, 5, false),
    ///         ReportColumn::new(Alignment::Right, 6, true),
    ///         ReportColumn::new(Alignment::Left, 0, true),
    ///     ]
    /// );
    /// ```
    #[macro_export]
    macro_rules! rptcols {
            // the terminal rule that creates the vector of column descriptions
            (@rc () -> [$($col_descrs:tt)*]) => {
                ::std::vec!($($col_descrs)*)
            };
            // from comma delimited markup creates a left justified, auto-sizing column
            (@rc (<, $($cols_markup:tt)*) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc ($($cols_markup)*) -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Left, 0, false)
                    ,
                ])
            };
            // creates a left justified auto-sizing column and ends markup parsing
            (@rc (<) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc () -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Left, 0, false)
                    ,
                ])
            };
            // from comma delimited markup creates a left justified, fixed width column
            (@rc (<=( $width:expr ), $($cols_markup:tt)*) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc ($($cols_markup)*) -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Left, $width, true)
                    ,
                ])
            };
            // creates a left justified, fixed width and ends markup parsing
            (@rc (<=( $width:expr ) ) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc () -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Left, $width, true)
                    ,
                ])
            };
            // from comma delimited markup creates a left justified, minimum width, auto-sizing column
            (@rc (<+( $width:expr ), $($cols_markup:tt)*) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc ($($cols_markup)*) -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Left, $width, false)
                    ,
                ])
            };
            // creates a left justified, minimum width, auto-sizing column and ends markup parsing
            (@rc (<+( $width:expr )) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc () -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Left, $width, false)
                    ,
                ])
            };
            // from comma delimited markup creates a center justified, auto-sizing column
            (@rc (^, $($cols_markup:tt)*) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc ($($cols_markup)*) -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Center, 0, false)
                    ,
                ])
            };
            // creates a center justified, auto-sizing column and ends markup parsing
            (@rc (^) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc () -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Center, 0, false)
                    ,
                ])
            };
            // from comma delimited markup creates a center justified, fixed width column
            (@rc (^=( $width:expr ), $($cols_markup:tt)*) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc ($($cols_markup)*) -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Center, $width, true)
                    ,
                ])
            };
            // creates a center justified, fixed width column and ends markup parsing
            (@rc (^=( $width:expr )) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc () -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Center, $width, true)
                    ,
                ])
            };
            // from comma delimited markup creates a center justified, minimum width, auto-sizing column
            (@rc (^+( $width:expr ), $($cols_markup:tt)*) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc ($($cols_markup)*) -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Center, $width, false)
                    ,
                ])
            };
            // creates a center justified, minimum width, auto-sizing column and ends markup parsing
            (@rc (^+( $width:expr )) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc () -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Center, $width, false)
                    ,
                ])
            };
            // from comma delimited markup creates a right justified, auto-sizing column
            (@rc (>, $($cols_markup:tt)*) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc ($($cols_markup)*) -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Right, 0, false)
                    ,
                ])
            };
            // creates a right justified, auto-sizing column and ends markup parsing
            (@rc (>) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc () -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Right, 0, false)
                    ,
                ])
            };
            // from comma delimited markup creates a right justified, fixed width column
            (@rc (>=( $width:expr ), $($cols_markup:tt)*) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc ($($cols_markup)*) -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Right, $width, true)
                    ,
                ])
            };
            // creates a right justified, fixed width column and ends markup parsing
            (@rc (>=( $width:expr )) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc () -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Right, $width, true)
                    ,
                ])
            };
            // from comma delimited markup creates a right justified, minimum width, auto-sizing column
            (@rc (>+( $width:expr ), $($cols_markup:tt)*) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc ($($cols_markup)*) -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Right, $width, false)
                    ,
                ])
            };
            // creates a right justified, minimum width, auto-sizing column and ends markup parsing
            (@rc (>+( $width:expr )) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc () -> [
                    $($col_descrs)*
                    $crate::text::ReportColumn::new($crate::text::Alignment::Right, $width, false)
                    ,
                ])
            };
            // creates a left justified as is text column
            (@rc (=, $($cols_markup:tt)*) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc ($($cols_markup)*) -> [
                    $($col_descrs)*
                    // $crate::text::ReportColumn::as_is()
                    $crate::text::ReportColumn::new($crate::text::Alignment::Left, 0, true)
                    ,
                ])
            };
            // creates a left justified, as is text column
            (@rc (=) -> [ $($col_descrs:tt)* ]) => {
                rptcols!(@rc () -> [
                    $($col_descrs)*
                    // $crate::text::ReportColumn::as_is()
                    $crate::text::ReportColumn::new($crate::text::Alignment::Left, 0, true)
                    ,
                ])
            };
            ($($col_markups:tt)*) => {
                rptcols!(@rc ($($col_markups)*) -> [])
            };
        }

    #[cfg(test)]
    mod tests {
        use super::super::{Alignment, ReportColumn, ReportData};
        #[test]
        fn data_cells() {
            assert_eq!(
                rptrow!("default", "alignment"),
                vec![ReportData::new("default", None), ReportData::new("alignment", None),]
            );
            assert_eq!(
                rptrow!(< "left", < "alignment"),
                vec![
                    ReportData::new("left", Some(Alignment::Left)),
                    ReportData::new("alignment", Some(Alignment::Left)),
                ]
            );
            assert_eq!(
                rptrow!(^ "center", ^ "alignment"),
                vec![
                    ReportData::new("center", Some(Alignment::Center)),
                    ReportData::new("alignment", Some(Alignment::Center)),
                ]
            );
            assert_eq!(
                rptrow!(> "align", > "right"),
                vec![
                    ReportData::new("align", Some(Alignment::Right)),
                    ReportData::new("right", Some(Alignment::Right)),
                ]
            );
        }
        #[test]
        fn column_formats() {
            assert_eq!(
                rptcols!(<, <),
                vec![ReportColumn::new(Alignment::Left, 0, false), ReportColumn::new(Alignment::Left, 0, false),]
            );
            assert_eq!(
                rptcols!(<+(10), <+(20)),
                vec![ReportColumn::new(Alignment::Left, 10, false), ReportColumn::new(Alignment::Left, 20, false),]
            );
            assert_eq!(
                rptcols!(<=(25), <=(50)),
                vec![ReportColumn::new(Alignment::Left, 25, true), ReportColumn::new(Alignment::Left, 50, true),]
            );
            assert_eq!(
                rptcols!(^, ^),
                vec![ReportColumn::new(Alignment::Center, 0, false), ReportColumn::new(Alignment::Center, 0, false),]
            );
            assert_eq!(
                rptcols!(^+(15), ^+(25)),
                vec![ReportColumn::new(Alignment::Center, 15, false), ReportColumn::new(Alignment::Center, 25, false),]
            );
            assert_eq!(
                rptcols!(^=(21), ^=(31)),
                vec![ReportColumn::new(Alignment::Center, 21, true), ReportColumn::new(Alignment::Center, 31, true),]
            );
            assert_eq!(
                rptcols!(>, >),
                vec![ReportColumn::new(Alignment::Right, 0, false), ReportColumn::new(Alignment::Right, 0, false),]
            );
            assert_eq!(
                rptcols!(>+(15), >+(25)),
                vec![ReportColumn::new(Alignment::Right, 15, false), ReportColumn::new(Alignment::Right, 25, false),]
            );
            assert_eq!(
                rptcols!(>=(21), >=(31)),
                vec![ReportColumn::new(Alignment::Right, 21, true), ReportColumn::new(Alignment::Right, 31, true),]
            );
            assert_eq!(
                rptcols!(=, =),
                vec![ReportColumn::new(Alignment::Left, 0, true), ReportColumn::new(Alignment::Left, 0, true)]
            )
        }
    }
}
