from tkinter import Canvas
from typing import Iterable

from .cell import Cell
from .data import Data
from .row import Row
from .text import Text
from ...infrastructure import Stopwatch
from ....config import get_logger

__all__ = ['Renderer']
log = get_logger(__name__)


class Renderer:
    def __init__(self, data: Data):
        self._data = data

    def add_column(self, canvas: Canvas, column: int, x: int):
        """Add a column to the canvas at the x offset."""
        stopwatch = Stopwatch()
        view = self._data.view
        if column >= view.columns:
            raise IndexError(f'Column {column} is out of range.')

        # set up the renderer
        width_px = view.column_width_px(column)
        y = 0
        for heading in self._data.headings:
            height_px = view.heading_height_px(heading.index)
            # special case the left top header cell when it is a span
            if heading.index == 0 and column == view.rect.left and self.is_header_left_top_span():
                self.render_header_left_top_span(canvas)
            else:
                self._render_cell(canvas, heading[column], x, y, width_px, height_px)
            y += height_px
        for content in self._data.contents[view.rect.top: view.rect.bottom]:
            height_px = view.content_height_px(content.index)
            self._render_cell(canvas, content[column], x, y, width_px, height_px)
            y += height_px
        log.info('added column %s %s', column, stopwatch)

    def append_columns(self, canvas: Canvas, width_px: int):
        """Add columns to the canvas until the view is full."""
        view = self._data.view
        while view.is_next_column_visible(width_px):
            self.add_column(canvas, view.rect.right, view.width_px())
            view.rect.width += 1

    def trim_columns(self, canvas: Canvas, width_px: int):
        """Remove columns from the canvas that are not visible."""
        stopwatch = Stopwatch()
        view = self._data.view

        # check the columns from right to left
        for right in reversed(range(view.rect.left, view.rect.right)):
            # always leave 1 column
            if view.rect.width < 2:
                break
            # delete columns until the view fits the window
            if view.is_last_column_visible(width_px):
                break
            tag = view.col_tag(right)
            log.debug('deleting %s', tag)
            canvas.delete(tag)
            view.rect.width -= 1
        log.info('trim_columns %s', stopwatch)

    def append_rows(self, canvas: Canvas, height_px: int):
        """Add rows to the canvas until the view is full."""
        view = self._data.view
        while view.is_next_row_visible(height_px):
            row = self._data.contents[view.rect.bottom]
            self.render_row(canvas, row, view.height_px())
            view.rect.height += 1

    def trim_rows(self, canvas: Canvas, height_px: int):
        """Remove rows from the canvas that are not currently visible."""
        stopwatch = Stopwatch()
        view = self._data.view
        # start at the bottom row and work your way up
        for bottom in reversed(range(view.rect.top, view.rect.bottom)):
            # always leave a row in the view
            if view.rect.height < 2:
                break
            # if the last row is visible you're done
            if view.is_last_row_visible(height_px):
                break
            tag = view.row_tag(bottom)
            canvas.delete(tag)
            view.rect.height -= 1
        log.debug('trim_rows %s', stopwatch)

    def refresh_view(self, canvas: Canvas):
        stopwatch = Stopwatch()
        y_offset = 0
        view = self._data.view
        for row in self._data.headings:
            self.render_row(canvas, row, y_offset)
            y_offset += view.heading_height_px(row.index)
        for row in self._data.contents[self._data.view.rect.top: self._data.view.rect.bottom]:
            self.render_row(canvas, row, y_offset)
            y_offset += view.content_height_px(row.index)
        log.info('refresh_view %s', stopwatch)

    def is_header_left_top_span(self) -> bool:
        """Check if the left top header cell is a span."""
        if len(self._data.headings) > 0:
            return self._data.headings[0][self._data.view.rect.left].text.is_span()
        return False

    def render_header_left_top_span(self, canvas: Canvas):
        """Special case rendering the left top header when it is a span."""
        if not self.is_header_left_top_span():
            log.error('Yikes... Not header left top span!')
            return
        row = self._data.headings[0]
        view = self._data.view
        x = view.x_offset
        left = view.rect.left
        # log.debug(' '.join([f'{i}[{view.column_width_px(i)}]' for i in range(view.columns)]))
        # log.debug(f'x={x} left={left} width={view.column_width_px(left)}')
        # move left to find the column span cell
        for span_col in reversed(range(1 if view.is_labeled else 0, left)):
            # the left hand side needs to be adjusted
            left -= 1
            span_width_px = view.column_width_px(span_col)
            x -= span_width_px
            y = 0
            span_cell = row[span_col]
            # log.debug(f'span_col={span_col} {cell} span_width={span_width_px} x={x} left={left}')
            # check to see if it is the column span cell
            if span_cell.text.is_type(Text.COLUMN_SPAN):
                # get the width of the column span cell
                span_width_px = view.cell_width_px(span_cell)
                span_height_px = view.heading_height_px(row.index)
                # the column span cell needs to reflect the view left hand side
                header_cell = span_cell.at(view.rect.left, row.index)
                # the span cell will be rendered at the adjusted position
                tags = self._cell_tags(header_cell)
                # log.debug(f'{header_cell} x={x} y={y} width={span_width_px} height={span_height_px} tags={tags}')
                header_cell.render(canvas, x, y, span_width_px, span_height_px, tags)
                canvas.tag_raise(view.label_tag, view.col_tag(header_cell.column))
                break

    def render_row(self, canvas: Canvas, row: Row, y_offset: int):
        """Add a row to the canvas."""
        # set up the renderer
        view = self._data.view
        x_offset = 0
        height = view.heading_height_px(row.index) if row.is_header else view.content_height_px(row.index)

        def render(c: Cell):
            nonlocal x_offset
            width = view.column_width_px(c.column)
            self._render_cell(canvas, c, x_offset, y_offset, width, height)
            x_offset += width

        # row labels need to be special cased otherwise it's just content
        if view.is_labeled:
            render(row[0])
        left = view.rect.left
        # check if the row is a header and the left top cell is a span
        if row.is_header and row.index == 0 and self.is_header_left_top_span():
            self.render_header_left_top_span(canvas)
            x_offset += view.column_width_px(left)
            left += 1
        for cell_ in row[left:view.rect.right]:
            render(cell_)

    def _render_cell(self, canvas: Canvas, cell_: Cell, x, y: int, width: int, height: int):
        """Set up the rendering of a cell."""
        view = self._data.view
        text = cell_.text
        if text.is_type(Text.HEADER) and text.is_type(Text.ROW_SPAN):
            height += sum([view.heading_height_px(i) for i in range(cell_.row + 1, cell_.row + text.span)])
        if text.is_type(Text.COLUMN_SPAN):
            width += sum([view.column_width_px(i) for i in range(cell_.column + 1, cell_.column + text.span)])
        tags = self._cell_tags(cell_)
        cell_.render(canvas, x, y, width, height, tags)

    def _cell_tags(self, cell_: Cell) -> Iterable[str]:
        """Create the tags associated with the cell."""
        view = self._data.view
        text = cell_.text
        # always label the column
        tags = [view.col_tag(cell_.column)]
        # labels don't h scroll
        if text.is_type(Text.LABEL):
            tags.append(view.label_tag)
        else:
            tags.append(view.hscroll_tag)
        # headers don't v scroll
        if text.is_type(Text.HEADER):
            tags.append(view.header_tag(cell_.row))
        else:
            tags.append(view.row_tag(cell_.row))
            tags.append(view.vscroll_tag)
        return tags
