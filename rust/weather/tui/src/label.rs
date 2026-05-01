//! The terminal UI label control.
//!
//! The [Label] provides a read-only text area.
//! 
use ratatui::prelude::{Alignment, Buffer, Line, Rect, Span, Style, Widget};

/// The label selector value when it has not been set.
///
pub const NO_SELECTOR: char = '\0';

/// The styling of a label.
///
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LabelStyles {
    /// The text style.
    pub text: Style,
    /// The selector text style.
    pub selector: Style,
}
impl LabelStyles {
    /// The [Label] default active styles.
    ///
    pub fn active() -> Self {
        Self { text: Style::default().green(), selector: Style::default().light_green().underlined() }
    }

    /// The [Label] default inactive styles.
    ///
    pub fn inactive() -> Self {
        Self { text: Style::default(), selector: Style::default().underlined() }
    }

    /// The [Label] default readonly styles.
    ///
    pub fn readonly() -> Self {
        Self { text: Style::default(), selector: Style::default() }
    }

    /// The [Label] default dim styles.
    ///
    pub fn dim() -> Self {
        Self { text: Style::default().dim(), selector: Style::default().dim().underlined() }
    }
}

/// The text label properties.
///
#[derive(Debug, Default)]
pub struct Label {
    /// The textual description of the label.
    text: String,
    /// The text alignment when rendering.
    align: Alignment,
    /// A character in the description that can select the label.
    selector: char,
    /// The label width.
    width: u16,
    /// The styles to use when rendering the label.
    styles: LabelStyles,
}
impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let alignment = match self.align {
            Alignment::Left => '<',
            Alignment::Center => '^',
            Alignment::Right => '>',
        };
        write!(f, "Label{}{}:{alignment}{}{}", '{', self.text, self.width, '}')
    }
}
impl Label {
    /// Create a new instance of the label from the text.
    ///
    /// # Arguments
    ///
    /// * `text` is the label text that will be rendered.
    ///
    pub fn new(text: impl ToString) -> Self {
        let text = text.to_string();
        let width = text.chars().count() as u16;
        Self { text, width, selector: NO_SELECTOR, ..Self::default() }
    }

    /// A builder method to set the alignment of the label text when rendered.
    ///
    /// # Arguments
    ///
    /// * `alignment` determines how the text will be justified when rendered.
    ///
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.align = alignment;
        self
    }

    /// Set the alignment that will be used when rendering the label. The [EditorGroup](super::editor_group::EditorGroup)
    /// uses this method to align all the labels within the group.
    ///
    /// # Arguments
    ///
    /// * `alignment` is how the text will be justified when rendered.
    ///
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.align = alignment;
    }

    /// A builder method that sets the width of the label.
    ///
    /// # Arguments
    ///
    /// * `width` sets the size of the label text regardless of the actual size.
    ///
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Set the width of the label.
    ///
    /// # Arguments
    ///
    /// * `width` sets the size of the label text regardless of the actual size.
    ///
    pub fn set_width(&mut self, width: u16) {
        self.width = width;
    }

    /// A builder method that sets the label selector character.
    ///
    /// # Arguments
    ///
    /// - `selector` is the label selector.
    ///
    pub fn with_selector(mut self, selector: char) -> Self {
        self.selector = selector;
        self
    }

    /// Get the label selector character.
    ///
    pub fn selector(&self) -> char {
        self.selector
    }

    /// A builder method that sets the initial label styles.
    ///
    /// # Arguments
    ///
    /// * `styles` will be used the next time the label is rendered.
    ///
    pub fn with_styles(mut self, styles: LabelStyles) -> Self {
        self.styles = styles;
        self
    }

    /// Unit tests need to peek at the label styles.
    ///
    #[cfg(test)]
    pub fn styles(&self) -> LabelStyles {
        self.styles
    }

    /// Unit tests need to peek at the alignment.
    ///
    #[cfg(test)]
    pub fn alignment(&self) -> Alignment {
        self.align
    }

    /// Get the size of the label.
    ///
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Set the label styles.
    ///
    /// # Arguments
    ///
    /// * `styles` will be used the next time the label is rendered.
    ///
    pub fn set_styles(&mut self, styles: LabelStyles) {
        self.styles = styles;
    }

    /// Draw the label on the terminal screen.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the checkbox will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    ///
    pub fn render(&self, area: Rect, buffer: &mut Buffer) {
        let text = self.text.as_str();
        match self.selector == NO_SELECTOR {
            true => Line::styled(text, self.styles.text).alignment(self.align).render(area, buffer),
            false => match text.split_once(self.selector) {
                None => Line::styled(text, self.styles.text).alignment(self.align).render(area, buffer),
                Some((lhs, rhs)) => {
                    let mut spans: Vec<Span> = Vec::with_capacity(3);
                    if lhs.len() > 0 {
                        spans.push(Span::styled(lhs, self.styles.text));
                    }
                    spans.push(Span::styled(self.selector.to_string(), self.styles.selector));
                    if rhs.len() > 0 {
                        spans.push(Span::styled(rhs, self.styles.text))
                    }
                    Line::from(spans).alignment(self.align).render(area, buffer);
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn alignment() {
        // labels are left aligned by default
        let testcase = Label::default();
        assert_eq!(testcase.align, Alignment::Left);

        // initialize the alignment
        let mut testcase = Label::new("foobar").with_alignment(Alignment::Right);
        assert_eq!(testcase.align, Alignment::Right);

        // modify the alignment
        testcase.set_alignment(Alignment::Center);
        assert_eq!(testcase.align, Alignment::Center);
    }

    #[test]
    fn init() {
        // default
        let testcase = Label::default();
        assert_eq!(testcase.text, "");
        assert_eq!(testcase.align, Alignment::Left);
        assert_eq!(testcase.selector, NO_SELECTOR);
        assert_eq!(testcase.styles.text, Style::default());
        assert_eq!(testcase.styles.selector, Style::default());
        assert_eq!(testcase.width(), 0);

        // from str
        let testcase = Label::new("foobar");
        assert_eq!(testcase.text, "foobar");
        assert_eq!(testcase.align, Alignment::Left);
        assert_eq!(testcase.selector, NO_SELECTOR);
        assert_eq!(testcase.styles.text, Style::default());
        assert_eq!(testcase.styles.selector, Style::default());
        assert_eq!(testcase.width(), 6);

        // from String
        let testcase = Label::new("testcase".to_string());
        assert_eq!(testcase.text, "testcase");
        assert_eq!(testcase.align, Alignment::Left);
        assert_eq!(testcase.selector, NO_SELECTOR);
        assert_eq!(testcase.styles.text, Style::default());
        assert_eq!(testcase.styles.selector, Style::default());
        assert_eq!(testcase.width(), 8);
    }

    #[test]
    fn selector() {
        let testcase = Label::default().with_selector('C');
        assert_eq!(testcase.selector(), 'C');
    }

    #[test]
    fn styles() {
        // styles
        let styles = LabelStyles { text: Style::default().red(), selector: Style::default().green() };
        let mut testcase = Label::default().with_styles(styles.clone());
        // testcase.styles(styles.clone());
        assert_eq!(testcase.styles.text, styles.text);
        assert_eq!(testcase.styles.selector, styles.selector);
        let set_styles = LabelStyles { text: Style::default().green(), selector: Style::default().red() };
        testcase.set_styles(set_styles.clone());
        assert_eq!(testcase.styles.text, set_styles.text);
        assert_eq!(testcase.styles.selector, set_styles.selector);
    }

    #[test]
    fn width() {
        // initialize the width
        let mut testcase = Label::new("test").with_width(8);
        assert_eq!(testcase.width(), 8);

        // modify the width
        testcase.set_width(6);
        assert_eq!(testcase.width(), 6);
    }

    #[test]
    fn render() {
        let area = Rect::new(0, 0, 5, 1);
        let mut buffer = Buffer::empty(area.clone());
        macro_rules! assert_cell {
            ($x: literal, $value:expr, $style:expr) => {
                assert_eq!(buffer[($x, 0)].symbol(), $value);
                assert_eq!(buffer[($x, 0)].style(), $style);
            };
        }

        // capture a buffer cell that does not have content
        let default_buffer_cell = &buffer[(0, 0)].clone();

        // these styles conform to the cell style in the buffer
        let styles = LabelStyles {
            text: Style::default().red().bg(Color::Reset).underline_color(Color::Reset),
            selector: Style::default().green().bg(Color::Reset).underline_color(Color::Reset),
        };

        // check the default render
        Label::default().render(area, &mut buffer);
        assert_cell!(0, " ", default_buffer_cell.style());
        assert_cell!(1, " ", default_buffer_cell.style());
        assert_cell!(2, " ", default_buffer_cell.style());
        assert_cell!(3, " ", default_buffer_cell.style());
        assert_cell!(4, " ", default_buffer_cell.style());

        let text = "abc";

        // lhs
        buffer.reset();
        Label::new(text).with_selector('a').with_styles(styles.clone()).render(area.clone(), &mut buffer);
        assert_cell!(0, "a", styles.selector);
        assert_cell!(1, "b", styles.text);
        assert_cell!(2, "c", styles.text);
        assert_cell!(3, " ", default_buffer_cell.style());
        assert_cell!(4, " ", default_buffer_cell.style());

        // centered
        buffer.reset();
        Label::new(text)
            .with_alignment(Alignment::Center)
            .with_selector('b')
            .with_styles(styles)
            .render(area, &mut buffer);
        assert_cell!(0, " ", default_buffer_cell.style());
        assert_cell!(1, "a", styles.text);
        assert_cell!(2, "b", styles.selector);
        assert_cell!(3, "c", styles.text);
        assert_cell!(4, " ", default_buffer_cell.style());

        // rhs
        buffer.reset();
        Label::new(text)
            .with_alignment(Alignment::Right)
            .with_selector('c')
            .with_styles(styles)
            .render(area, &mut buffer);
        assert_cell!(0, " ", default_buffer_cell.style());
        assert_cell!(1, " ", default_buffer_cell.style());
        assert_cell!(2, "a", styles.text);
        assert_cell!(3, "b", styles.text);
        assert_cell!(4, "c", styles.selector);
    }
}
