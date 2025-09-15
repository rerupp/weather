//! The report viewer UI.
//!
//! The [ReportView] is used to display a `toolslib` `ReportSheet` in a scrollable
//! window. The `ReportView` can be configured to have an identifier. It supports
//! scrolling in both horizontal and vertical directions.
//!
use super::*;
use ratatui::symbols::{block, scrollbar};
use std::cell::RefCell;
use styles::{CatalogType, StyleCatalog, StyleId};
use toolslib::report::*;

/// The metadata used to draw the visible report page.
///
#[derive(Default, Debug)]
struct Page {
    /// Identifies if the list is active or not.
    active: bool,
    /// The size of the list area from the last render.
    render_size: Size,
    /// The size of the contents area from the last render.
    contents_size: Size,
    // The first visible row in the list.
    row_offset: usize,
    /// The current row in the list.
    selected_row: usize,
    /// This will be true if render added a vertical scrollbar.
    v_scrollbar: bool,
    /// The separator between list columns.
    separator: String,
    /// Show which row in the list is currently selected.
    show_selected: bool,
    /// Keeps the first column visible during horizontal scrolling.
    column_labels: bool,
    /// The first left side visible column
    column_offset: usize,
    /// Allows the list to scroll horizontally.
    h_scroll: bool,
}

/// The metadata holding the content of a report row column.
///
#[derive(Debug)]
struct ReportCell {
    /// The text that will be drawn.
    text: String,
    /// Use the reports cell type to identify the type of report content.
    cell_type: CellType,
    /// Use the reports cell layout to draw the content.
    cell_layout: CellLayout,
}
/// Convert the report [cell data](CellData) into a [report cell](ReportCell).
impl From<CellData<'_>> for ReportCell {
    fn from(cell_data: CellData) -> Self {
        Self { text: cell_data.to_string(), cell_type: cell_data.cell_type, cell_layout: cell_data.layout }
    }
}
impl ReportCell {
    /// Query the content to see if it is a header.
    fn is_header(&self) -> bool {
        self.cell_type == CellType::Header
    }
    /// Get the width of the contents.
    fn width(&self) -> u16 {
        self.cell_layout.width() as u16
    }
}

/// The metadata holding a row of report data.
#[derive(Debug)]
struct ReportRow {
    /// The collection of report cells that make up the report row.
    cells: Vec<ReportCell>,
    /// The overall width of the report row.
    cell_widths: Vec<u16>,
}
/// Convert the report [row](SheetRow) into a [report row](ReportRow).
impl From<SheetRow<'_>> for ReportRow {
    fn from(sheet_row: SheetRow<'_>) -> Self {
        let cells: Vec<ReportCell> = sheet_row.into_iter().map(|cell_data| cell_data.into()).collect();
        let cell_widths = cells.iter().map(|column| column.width()).collect();
        Self { cells, cell_widths }
    }
}
impl ReportRow {
    /// Query the row to see if all cells are headers.
    ///
    fn is_header(&self) -> bool {
        self.cells.iter().find_map(|cell| if cell.is_header() { None } else { Some(()) }).is_none()
    }
    /// Get the width of the row.
    ///
    /// # Arguments
    ///
    /// - `page` is used to get the length of the column separator.
    ///
    fn width(&self, page: &Page) -> u16 {
        let cells_width = self.cell_widths.iter().sum::<u16>() as usize;
        let separator_width = self.cells.len().saturating_sub(1) * page.separator.len();
        (cells_width + separator_width) as u16
    }
    /// Create a ratatui [Line] that can be drawn on the terminal screen.
    ///
    /// # Arguments
    ///
    /// - `page` provides the metadata needed to convert the row into something that can be drawn.
    /// - `text_style` is used to draw report text content.
    /// - `header_style` is used to draw report header content.
    ///
    fn to_line<'l>(&'l self, page: &'l Page, (text_style, header_style): (Style, Style)) -> Line<'l> {
        let mut spans = vec![];
        let mut spans_len = 0u16;
        macro_rules! add_span {
            ($cell:expr) => {
                spans_len += $cell.cell_layout.width() as u16;
                match $cell.cell_type {
                    CellType::Header => spans.push(Span::styled($cell.text.as_str(), header_style)),
                    CellType::Text | CellType::Plain => spans.push(Span::styled($cell.text.as_str(), text_style)),
                    _ => (),
                }
            };
        }
        let separator_len = page.separator.len() as u16;
        for (column, cell) in self.cells.iter().enumerate() {
            if spans_len > page.render_size.width {
                break;
            }
            if page.column_labels && column == 0 {
                add_span!(cell);
                continue;
            }
            if page.column_offset > column {
                continue;
            }
            if spans.len() > 0 {
                spans_len += separator_len;
                spans.push(Span::styled(&page.separator, text_style));
            }
            add_span!(cell);
        }
        Line::from(spans)
    }
}

/// The report headers metadata.
#[derive(Debug, Default)]
struct HeaderRows {
    /// The collection of header report rows.
    rows: Vec<ReportRow>,
    /// The maximum number of header columns in the report.
    max_columns: usize,
    /// The maximum header report row width.
    max_width: u16,
}
impl HeaderRows {
    /// Add a row to the header collection.
    ///
    /// # Arguments
    ///
    /// - `row` is the header row that will be added.
    /// - `page` provide the metadata required to get the row width.
    ///
    fn add_row(&mut self, row: ReportRow, page: &Page) {
        self.max_columns = cmp::max(self.max_columns, row.cells.len());
        self.max_width = cmp::max(self.max_width, row.width(page));
        self.rows.push(row);
    }
    /// Draw the headers on the terminal screen.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the rows will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` contains the [styles](StyleCatalog) used the draw the row.
    /// - `page` provides the metadata required to draw the header rows.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer, styles: &StyleCatalog, page: &Page) {
        let styles = (styles.get(StyleId::Text), styles.get(StyleId::Header));
        let header_rows = self.rows.iter().map(|row| row.to_line(page, styles));
        Widget::render(List::default().items(header_rows), area, buffer);
    }
}

/// The report contents metadata.
#[derive(Debug, Default)]
struct ContentRows {
    /// The collection of report contents (not headers).
    rows: Vec<ReportRow>,
    /// The maximum number of columns in the contents.
    max_columns: usize,
    /// The maximum report row width.
    max_width: u16,
}
impl ContentRows {
    /// Add a row to the content collection.
    ///
    /// # Arguments
    ///
    /// - `row` is the content row that will be added.
    /// - `page` provide the metadata required to get the row width.
    ///
    fn add_row(&mut self, row: ReportRow, page: &Page) {
        self.max_columns = cmp::max(self.max_columns, row.cells.len());
        self.max_width = cmp::max(self.max_width, row.width(page));
        self.rows.push(row);
    }
    /// Draw the content on the terminal screen.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the rows will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` contains the [styles](StyleCatalog) used the draw the row.
    /// - `page` provides the metadata required to draw the content rows.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer, styles: &StyleCatalog, page: &Page) -> Option<Position> {
        if self.rows.is_empty() {
            None?;
        }
        // create a window of list content
        let normal = (styles.get(StyleId::Text), styles.get(StyleId::Header));
        let selected = (styles.get(StyleId::Highlight), styles.get(StyleId::Header));
        let render_height = area.height as usize;
        let mut lines = Vec::with_capacity(render_height);
        for (index, row) in self.rows.iter().enumerate() {
            // if the render area is full the list window is complete
            if lines.len() >= render_height {
                break;
            }
            // make sure you are at or past the top of the render window
            if index < page.row_offset {
                continue;
            }
            lines.push(match page.show_selected && page.selected_row == index {
                true => row.to_line(&page, selected),
                false => row.to_line(&page, normal),
            });
        }
        // show the list
        let list_area = match page.v_scrollbar {
            true => inner_rect(area, (0, 0), (-2, 0)),
            false => area,
        };
        let list = List::default().items(lines);
        Widget::render(list, list_area, buffer);
        if page.v_scrollbar {
            let scrollbar_area = inner_rect(area, (-1, 0), (0, 0));
            let mut state = ScrollbarState::default()
                .content_length(self.rows.len())
                .viewport_content_length(area.height as usize)
                .position(page.selected_row);
            Scrollbar::new(ScrollbarOrientation::VerticalLeft)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(None)
                .end_symbol(None)
                .thumb_symbol(block::FULL)
                .style(styles.get(StyleId::Scrollbar))
                .render(scrollbar_area, buffer, &mut state);
        }
        let y_coord = area.y + (page.selected_row - page.row_offset) as u16;
        Some(Position::new(area.x, y_coord))
    }
}

/// The metadata that tracks the horizontal position of a column on the terminal screen.
#[derive(Debug)]
struct ColumnMap {
    /// The x offset of the column on the terminal screen.
    offset: u16,
    /// The width of the column.
    width: u16,
}
#[derive(Debug, Default)]
struct ColumnMaps(Vec<ColumnMap>);
impl ColumnMaps {
    /// Get a columns mapping.
    ///
    /// # Arguments
    ///
    /// - `column` identifies what column mapping will be returned.
    ///
    fn get(&self, column: usize) -> &ColumnMap {
        self.0.get(column).expect("column is out of bounds...")
    }
    /// Get the last column mapping.
    ///
    fn get_last(&self) -> &ColumnMap {
        self.0.last().expect("ColumnMaps is empty...")
    }
    /// Get the maximum column offset.
    ///
    fn max_offset(&self) -> usize {
        self.0.len().saturating_sub(1)
    }
    /// Calculate the width of the report starting at a specific column.
    ///
    /// # Arguments
    ///
    /// `column` identifies what column is the left hand side of what will be drawn.
    ///
    fn width_from_offset(&self, column: usize) -> u16 {
        let lhs_column = self.0.get(column).expect("lhs column is out of bounds...");
        let last_column = self.get_last();
        let offset_width = (last_column.offset + last_column.width).saturating_sub(lhs_column.offset);
        offset_width
    }
}

macro_rules! log_page {
    ($what:expr, $page:expr) => {
        #[cfg(debug_assertions)]
        {
            // log::debug!("{} {:?}", $what, $page)
        }
    };
}

/// The [report sheet](ReportSheet) viewer.
#[derive(Debug)]
pub struct ReportView {
    /// The report view implements [Control] so support it having an identifier.
    id: Option<String>,
    /// The collection of report headers.
    headers: HeaderRows,
    /// The collection of report contents.
    contents: ContentRows,
    /// The size of the report.
    size: Size,
    /// The page metadata.
    page: RefCell<Page>,
    /// The report column map.
    column_maps: ColumnMaps,
    /// The report catalog type is [CatalogType::ReportView].
    pub catalog_type: CatalogType,
}
impl std::fmt::Display for ReportView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReportView[")?;
        if let Some(id) = self.id.as_ref() {
            write!(f, "{}", id)?;
        }
        write!(f, "] active={}", self.page.borrow().active)
    }
}
impl ReportView {
    /// Create the report view from a [report sheet](ReportSheet).
    ///
    /// # Arguments
    ///
    /// - `report` contains the content that will be drawn.
    /// - `separator` is an optional report column separator (default is ' ').
    ///
    pub fn new(report: ReportSheet, separator: Option<&str>) -> ReportView {
        // create the page metadata
        let mut page = Page::default();
        page.separator = separator.map_or(" ".to_string(), |s| s.to_string());
        let separator_len = page.separator.len() as u16;
        // get the column layouts
        let report_layouts = report.layouts();
        let mut column_maps = ColumnMaps::default();
        let mut offset = 0u16;
        for cell_layout in report_layouts {
            if column_maps.0.len() > 0 {
                offset += separator_len;
            }
            let cell_layout_width = cell_layout.width() as u16;
            column_maps.0.push(ColumnMap { offset, width: cell_layout_width });
            offset += cell_layout_width;
        }
        // create the list contents
        let mut headers = HeaderRows::default();
        let mut contents = ContentRows::default();
        for sheet_row in &report {
            let report_row: ReportRow = sheet_row.into();
            match report_row.is_header() {
                true => headers.add_row(report_row, &page),
                false => contents.add_row(report_row, &page),
            }
        }
        let height = (headers.rows.len() + contents.rows.len()) as u16;
        debug_assert!(height > 0);
        let width = cmp::max(headers.max_width, contents.max_width);
        let size = Size { width, height };
        Self {
            id: None,
            headers,
            contents,
            size,
            page: RefCell::new(page),
            column_maps,
            catalog_type: CatalogType::ReportView,
        }
    }
    /// A builder method that sets the reports identifier.
    ///
    /// # Arguments
    ///
    /// - `id` is the control identifier.
    ///
    pub fn with_id(mut self, id: impl ToString) -> Self {
        self.id.replace(id.to_string());
        self
    }
    /// A builder method that sets the report active state.
    ///
    /// # Arguments
    ///
    /// - `yes_no` is the active state.
    ///
    pub fn with_active(self, yes_no: bool) -> Self {
        self.page.borrow_mut().active = yes_no;
        self
    }
    /// A builder method that forces the current row in the report to be highlighted.
    ///
    /// # Arguments
    ///
    /// - `yes_no` determines if the current row will be highlighted.
    ///
    pub fn with_show_selected(self, yes_no: bool) -> Self {
        self.page.borrow_mut().show_selected = yes_no;
        self
    }
    /// A builder method that treats the left hand side column as labels.
    ///
    /// # Arguments
    ///
    /// - `yes_no` controls if the left hand side column should be treated labels.
    ///
    pub fn with_column_labels(self, yes_no: bool) -> Self {
        self.page.borrow_mut().column_labels = yes_no;
        self
    }
    /// A builder method that enables the report to scroll horizontally in case the terminal width is narrower than
    /// the report content.
    ///
    /// # Arguments
    ///
    /// - `yes_no` controls if the report can scroll horizontally.
    ///
    pub fn with_horizontal_scroll(self, yes_no: bool) -> Self {
        self.page.borrow_mut().h_scroll = yes_no;
        self
    }
    /// Get the index of the currently selected row.
    pub fn selected_row(&self) -> usize {
        self.page.borrow().selected_row
    }
    /// Set the report selected row to the first line of content. This will return [ControlResult::NotAllowed]
    /// if the first row is already selected otherwise [ControlResult::Continue] will be returned.
    fn move_first(&self) -> ControlFlow<ControlResult> {
        let mut page = self.page.borrow_mut();
        log_page!("move_first", page);
        match page.selected_row == 0 {
            true => break_event!(ControlResult::NotAllowed),
            false => {
                page.row_offset = 0;
                page.selected_row = page.row_offset;
                break_event!(ControlResult::Continue)
            }
        }
    }
    /// Set the report selected row to the last line of content. This will return [ControlResult::NotAllowed]
    /// if the last row is already selected otherwise [ControlResult::Continue] will be returned.
    fn move_last(&self) -> ControlFlow<ControlResult> {
        let mut page = self.page.borrow_mut();
        log_page!("move_last", page);
        let last_row = self.contents.rows.len().saturating_sub(1);
        match page.selected_row == last_row {
            true => break_event!(ControlResult::NotAllowed),
            false => {
                let page_rows = page.contents_size.height.saturating_sub(1) as usize;
                page.row_offset = last_row.saturating_sub(page_rows);
                page.selected_row = last_row;
                break_event!(ControlResult::Continue)
            }
        }
    }
    /// Set the report selected row to the previous line of content. This will return [ControlResult::NotAllowed]
    /// if the first row is already selected otherwise [ControlResult::Continue] will be returned.
    fn move_up(&self) -> ControlFlow<ControlResult> {
        let mut page = self.page.borrow_mut();
        log_page!("move_up", page);
        match page.selected_row == 0 {
            true => break_event!(ControlResult::NotAllowed),
            false => {
                page.selected_row -= 1;
                if page.selected_row < page.row_offset {
                    page.row_offset = page.selected_row;
                }
                break_event!(ControlResult::Continue)
            }
        }
    }
    /// Set the report selected row to the next line of content. This will return [ControlResult::NotAllowed]
    /// if the last row is already selected otherwise [ControlResult::Continue] will be returned.
    fn move_down(&self) -> ControlFlow<ControlResult> {
        let mut page = self.page.borrow_mut();
        log_page!("move_down", page);
        let last_row = self.contents.rows.len().saturating_sub(1);
        match page.selected_row == last_row {
            true => break_event!(ControlResult::NotAllowed),
            false => {
                page.selected_row = cmp::min(page.selected_row + 1, last_row);
                // check to see if the resulting page height exceeds the content height
                if (page.selected_row - page.row_offset) as u16 >= page.contents_size.height {
                    page.row_offset += 1;
                }
                break_event!(ControlResult::Continue)
            }
        }
    }
    /// Set the report selected row to the selected row + page size. This will return [ControlResult::NotAllowed]
    /// if the last row is already selected otherwise [ControlResult::Continue] will be returned.
    fn page_down(&self) -> ControlFlow<ControlResult> {
        let mut page = self.page.borrow_mut();
        log_page!("page_down", page);
        let last_row = self.contents.rows.len().saturating_sub(1);
        match page.selected_row == last_row {
            true => break_event!(ControlResult::NotAllowed),
            false => {
                let render_height = page.contents_size.height as usize;
                let page_rows = render_height.saturating_sub(1);
                let offset = page.row_offset + page_rows;
                // prevent the list from having blank lines
                if offset + render_height >= last_row {
                    page.selected_row = last_row;
                    page.row_offset = last_row - page_rows;
                } else {
                    page.row_offset = offset;
                    page.selected_row = page.row_offset;
                }
                break_event!(ControlResult::Continue)
            }
        }
    }
    /// Set the report selected row to the selected row - page size. This will return [ControlResult::NotAllowed]
    /// if the first row is already selected otherwise [ControlResult::Continue] will be returned.
    fn page_up(&self) -> ControlFlow<ControlResult> {
        let mut page = self.page.borrow_mut();
        log_page!("page_up", page);
        match page.selected_row == 0 {
            true => break_event!(ControlResult::NotAllowed),
            false => {
                // let page_rows = page.render_size.height.saturating_sub(1) as usize;
                let page_rows = page.contents_size.height.saturating_sub(1) as usize;
                let offset = page.row_offset.saturating_sub(page_rows);
                if offset == 0 {
                    page.selected_row = 0;
                    page.row_offset = 0;
                } else {
                    page.row_offset = offset;
                    page.selected_row = page.row_offset;
                }
                break_event!(ControlResult::Continue)
            }
        }
    }
    /// Set the report left hand side column to the current left hand side + 1. This will return
    /// [ControlResult::NotAllowed] if the right hand side of the report is already visible or
    /// horizontal scrolling is not enabled, otherwise [ControlResult::Continue] will be returned.
    fn move_left(&self) -> ControlFlow<ControlResult> {
        let mut page = self.page.borrow_mut();
        log_page!("move_left", page);
        match page.h_scroll {
            true => match page.column_offset == 0 {
                true => (),
                false => {
                    match page.column_labels && page.column_offset == 2 {
                        true => page.column_offset = 0,
                        false => page.column_offset -= 1,
                    }
                    break_event!(ControlResult::Continue)?;
                }
            },
            false => (),
        }
        break_event!(ControlResult::NotAllowed)
    }
    /// Set the report left hand side column to the current left hand side - 1. This will return
    /// [ControlResult::NotAllowed] if already at the left most report column or horizontal scrolling
    /// is not enabled, otherwise [ControlResult::Continue] will be returned.
    fn move_right(&self) -> ControlFlow<ControlResult> {
        // don't  bother if horizontal scrolling hasn't been set
        let mut page = self.page.borrow_mut();
        log_page!("move_right", page);
        match page.h_scroll {
            false => (),
            true => match self.column_maps.0.len() < 2 {
                true => (),
                false => {
                    let mut width = self.column_maps.width_from_offset(page.column_offset);
                    if page.column_labels && page.column_offset > 0 {
                        // take into account the label column
                        width += self.column_maps.get(1).offset;
                    }
                    match page.contents_size.width > width {
                        true => (),
                        // special case having column labels
                        false => {
                            match page.column_labels && page.column_offset == 0 {
                                true => page.column_offset = 2,
                                false => {
                                    page.column_offset = cmp::min(page.column_offset + 1, self.column_maps.max_offset())
                                }
                            }
                            break_event!(ControlResult::Continue)?;
                        }
                    }
                }
            },
        }
        break_event!(ControlResult::NotAllowed)
    }
    /// Set the report to the left hand side column. This will return [ControlResult::NotAllowed] if the
    /// first column already visible or horizontal scrolling is not enabled, otherwise
    /// [ControlResult::Continue] will be returned.
    fn move_lhs(&self) -> ControlFlow<ControlResult> {
        let mut page = self.page.borrow_mut();
        log_page!("move_lhs", page);
        match page.h_scroll {
            false => (),
            true => match page.column_offset == 0 {
                true => (),
                false => {
                    page.column_offset = 0;
                    break_event!(ControlResult::Continue)?;
                }
            },
        }
        break_event!(ControlResult::NotAllowed)
    }
    /// Set the report so the last column is visible. This will return [ControlResult::NotAllowed]
    /// if the last column is already visible or horizontal scrolling is not enabled, otherwise
    /// [ControlResult::Continue] will be returned.
    fn move_rhs(&self) -> ControlFlow<ControlResult> {
        let mut page = self.page.borrow_mut();
        log_page!("move_rhs", page);
        match page.h_scroll {
            false => (),
            // there's nothing to do if the last column is already visible
            true => match page.contents_size.width >= self.column_maps.width_from_offset(page.column_offset) {
                true => (),
                false => {
                    let mut column_offset = self.column_maps.max_offset();
                    while column_offset != 0 {
                        column_offset -= 1;
                        if page.contents_size.width >= self.column_maps.width_from_offset(column_offset) {
                            break;
                        }
                    }
                    page.column_offset = column_offset;
                    break_event!(ControlResult::Continue)?;
                }
            },
        }
        break_event!(ControlResult::NotAllowed)
    }
}
impl Control for ReportView {
    /// Get the report view control identifier.
    ///
    fn id(&self) -> &str {
        self.id.as_ref().map_or("", |id| id.as_str())
    }
    /// The report view selector will always be a null character ('\0').
    fn selector(&self) -> char {
        '\0'
    }
    /// Get the size of the report.
    fn size(&self) -> Size {
        let page = self.page.borrow();
        match page.render_size.height < self.size.height {
            true => Size { width: self.size.width + 2, height: self.size.height },
            false => self.size,
        }
    }
    /// Query if the report view is active or not.
    fn is_active(&self) -> bool {
        self.page.borrow().active
    }
    /// Set the report view active state.
    ///
    /// # Arguments
    ///
    /// - `yes_no` is used to set the active state.
    ///
    fn set_active(&mut self, yes_no: bool) {
        // self.active = yes_no;
        self.page.borrow_mut().active = yes_no;
    }
    /// Draw the report on the terminal screen.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the report will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` contains the [styles](StyleCatalog) used the draw the report.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer, styles: &StyleCatalog) -> Option<Position> {
        log_render!(self.to_string());
        // update the page before rendering the list
        let mut page = self.page.borrow_mut();
        page.v_scrollbar = area.height < (self.headers.rows.len() + self.contents.rows.len()) as u16;
        // page.h_scrollbar = area.width < cmp::max(self.headers.max_width, self.contents.max_width);
        page.render_size.height = area.height;
        page.render_size.width = area.width;
        // now the list can be rendered
        let headers_height = self.headers.rows.len() as i32;
        if headers_height > 0 {
            let headers_area = inner_rect(area, (0, 0), (0, headers_height));
            self.headers.render(headers_area, buffer, styles, &page);
        }
        let contents_area = inner_rect(area, (0, headers_height), (0, 0));
        page.contents_size.height = contents_area.height;
        page.contents_size.width = contents_area.width;
        self.contents.render(contents_area, buffer, styles, &mut &page)
    }
    /// Consume a key pressed event. The report will return [Continue](ControlFlow::Continue) if the event was
    /// not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a key pressed event.
    ///
    fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
        // if the list fits the last render height disallow scrolling
        let render_height = self.page.borrow().contents_size.height;
        log_key_pressed!(self.to_string());
        match render_height < 1 {
            // if the window cannot be drawn you need to continue so the dialog can check if it is a button action
            true => ControlFlow::Continue(()),
            false => match (key_event.modifiers, key_event.code) {
                // indicates the active row has been selected
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    break_event!(ControlResult::Selected(String::default()))
                }
                (KeyModifiers::NONE, KeyCode::Up) => self.move_up(),
                (KeyModifiers::NONE, KeyCode::Down) => self.move_down(),
                (KeyModifiers::NONE, KeyCode::PageDown) => self.page_down(),
                (KeyModifiers::NONE, KeyCode::PageUp) => self.page_up(),
                (KeyModifiers::NONE, KeyCode::Left) => self.move_left(),
                (KeyModifiers::NONE, KeyCode::Right) => self.move_right(),
                (KeyModifiers::NONE, KeyCode::Home) => self.move_lhs(),
                (KeyModifiers::NONE, KeyCode::End) => self.move_rhs(),
                (KeyModifiers::CONTROL, KeyCode::Home) => self.move_first(),
                (KeyModifiers::CONTROL, KeyCode::End) => self.move_last(),
                _ => ControlFlow::Continue(()),
            },
        }
    }
}
