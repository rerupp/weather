//! The command bar is similar in concept to the Python `textual` command pallet.

use ratatui::{
    layout::Alignment,
    prelude::{Buffer, Line, Rect, Span, Style, Widget},
};

/// The command bar styles establish how the command bar appears when being rendered.
#[derive(Debug, Default, Clone, Copy)]
pub struct CommandBarStyles {
    /// The command key style.
    pub key: Style,
    /// The command key description style.
    pub description: Style,
    /// The command separator style.
    pub separator: Style,
}
impl CommandBarStyles {
    pub fn common() -> Self {
        Self {
            key: Style::default().blue().bold(),
            description: Style::default().blue(),
            separator: Style::default().light_blue(),
        }
    }
}

/// The string to use as the command separator when it is rendered.
///
static SEPARATOR: &str = " | ";

/// The width of the command separator.
///
static SEPARATOR_WIDTH: u16 = 3;

/// The container for commands.
///
#[derive(Debug, Default)]
pub struct CommandBar {
    /// The commands consist of the command key and description of the key.
    commands: Vec<(String, String)>,
    /// The horizontal alignment of the command bar.
    align: Alignment,
    /// The styles used to render the command bar.
    styles: CommandBarStyles,
}
impl CommandBar {
    /// A builder method that adds a command key and description.
    ///
    /// # Arguments
    ///
    /// * `key` is the command key.
    /// * `description` describes the command key.
    ///
    pub fn add_command(mut self, key: impl ToString, description: impl ToString) -> Self {
        self.commands.push((key.to_string(), description.to_string()));
        self
    }

    /// A builder method that sets the alignment of the command bar within the rendered area.
    ///
    /// # Arguments
    ///
    /// * `styles` will be used when rendering the command bar.
    ///
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.align = alignment;
        self
    }

    /// A builder method that sets the style of the command bar when it is rendered.
    ///
    /// # Arguments
    ///
    /// * `styles` will be used when rendering the command bar.
    ///
    pub fn with_styles(mut self, styles: CommandBarStyles) -> Self {
        self.styles = styles;
        self
    }

    /// Calculate the terminal screen width needed to show the command bar.
    ///
    pub fn width(&self) -> u16 {
        // self.commands.iter().map(|(key, description) | key.len() + description.len()).sum()
        let mut width = 0;
        let mut iter = self.commands.iter();
        if let Some((key, description)) = iter.next() {
            width += key.len() + description.len();
            while let Some((key, description)) = iter.next() {
                width += key.len() + description.len() + SEPARATOR_WIDTH as usize;
            }
        }
        width as u16
    }

    /// Draw the command bar in the buffer.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the command bar will be drawn.
    /// - `buffer` is used to add the command bar area to the terminal screen.
    ///
    pub fn render(&self, area: Rect, buffer: &mut Buffer) {
        let mut spans = vec![];
        for (key, description) in self.commands.iter() {
            if spans.len() > 0 {
                spans.push(Span::styled(SEPARATOR, self.styles.separator));
            }
            spans.push(Span::styled(key, self.styles.key));
            spans.push(Span::styled(description, self.styles.description));
        }
        Line::from(spans).alignment(self.align).render(area, buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::Color;

    #[test]
    fn init() {
        let testcase = CommandBar::default().add_command("^A", " Add").add_command("^M", " Modify");
        assert_eq!(testcase.commands.len(), 2);
        assert_eq!(testcase.commands[0].0, "^A");
        assert_eq!(testcase.commands[0].1, " Add");
        assert_eq!(testcase.commands[1].0, "^M");
        assert_eq!(testcase.commands[1].1, " Modify");
    }

    #[test]
    fn width() {
        let testcase = CommandBar::default().add_command("^O", "One");
        assert_eq!(testcase.width(), 5);

        let testcase = CommandBar::default().add_command("^O", "One").add_command("^T", "Two");
        assert_eq!(testcase.width(), 5 + 5 + SEPARATOR_WIDTH);
    }

    #[test]
    fn render() {
        let mut testcase = CommandBar::default()
            .add_command("O", " o")
            .add_command("T", " t")
            .with_alignment(Alignment::Left)
            .with_styles(CommandBarStyles {
                key: Style::default().red().bg(Color::Reset).underline_color(Color::Reset),
                description: Style::default().green().bg(Color::Reset).underline_color(Color::Reset),
                separator: Style::default().blue().bg(Color::Reset).underline_color(Color::Reset),
            });

        let area = Rect { x: 0, y: 0, width: testcase.width() as u16 + 2, height: 1 };
        let mut buffer = Buffer::empty(area);
        let default_cell = &buffer[(0, 0)].clone();

        macro_rules! assert_cell {
            ($x: literal, $value:expr, $style:expr) => {
                assert_eq!(buffer[($x, 0)].symbol(), $value);
                assert_eq!(buffer[($x, 0)].style(), $style);
            };
        }

        // left aligned
        testcase.render(area, &mut buffer);
        assert_cell!(0, "O", testcase.styles.key);
        assert_cell!(1, " ", testcase.styles.description);
        assert_cell!(2, "o", testcase.styles.description);
        assert_cell!(3, " ", testcase.styles.separator);
        assert_cell!(4, "|", testcase.styles.separator);
        assert_cell!(5, " ", testcase.styles.separator);
        assert_cell!(6, "T", testcase.styles.key);
        assert_cell!(7, " ", testcase.styles.description);
        assert_cell!(8, "t", testcase.styles.description);
        assert_cell!(9, " ", default_cell.style());
        assert_cell!(10, " ", default_cell.style());

        // center aligned
        buffer.reset();
        testcase = testcase.with_alignment(Alignment::Center);
        testcase.render(area, &mut buffer);
        assert_cell!(0, " ", default_cell.style());
        assert_cell!(1, "O", testcase.styles.key);
        assert_cell!(2, " ", testcase.styles.description);
        assert_cell!(3, "o", testcase.styles.description);
        assert_cell!(4, " ", testcase.styles.separator);
        assert_cell!(5, "|", testcase.styles.separator);
        assert_cell!(6, " ", testcase.styles.separator);
        assert_cell!(7, "T", testcase.styles.key);
        assert_cell!(8, " ", testcase.styles.description);
        assert_cell!(9, "t", testcase.styles.description);
        assert_cell!(10, " ", default_cell.style());

        // right aligned
        buffer.reset();
        testcase = testcase.with_alignment(Alignment::Right);
        testcase.render(area, &mut buffer);
        assert_cell!(0, " ", default_cell.style());
        assert_cell!(1, " ", default_cell.style());
        assert_cell!(2, "O", testcase.styles.key);
        assert_cell!(3, " ", testcase.styles.description);
        assert_cell!(4, "o", testcase.styles.description);
        assert_cell!(5, " ", testcase.styles.separator);
        assert_cell!(6, "|", testcase.styles.separator);
        assert_cell!(7, " ", testcase.styles.separator);
        assert_cell!(8, "T", testcase.styles.key);
        assert_cell!(9, " ", testcase.styles.description);
        assert_cell!(10, "t", testcase.styles.description);
    }
}
