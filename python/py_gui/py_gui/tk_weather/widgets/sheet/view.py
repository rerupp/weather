from typing import List

from .cell import Cell
from .rect import Rect
from .text import Text

__all__ = ['View']


class View:
    def __init__(self, column_widths_px: List[int], heading_heights_px: List[int], view_heights_px: List[int],
                 view_labeled: bool):
        self._column_widths_px = column_widths_px
        self._columns = len(self._column_widths_px)
        # set up the view area
        self._heading_heights_px = heading_heights_px
        self._heading_rows = len(heading_heights_px)
        self._view_heights = view_heights_px
        self._view_rows = len(self._view_heights)
        self._view_x_offset = column_widths_px[0] if view_labeled else 0
        self._view_y_offset = sum([height for height in heading_heights_px])
        self._view_rect = Rect(left=1 if view_labeled else 0)
        self._view_labeled = view_labeled
        # the tag creation attributes
        self.col_tag = lambda c: f'col{c}'
        self.row_tag = lambda r: f'row{r}'
        self.header_tag = lambda h: f'hdr{h}'
        self.label_tag = 'label'
        self.hscroll_tag = 'hscroll'
        self.vscroll_tag = 'vscroll'

    def __str__(self):
        return f'{self._columns}x{self._view_rows}@{self._view_rect}'

    @property
    def rows(self):
        return self._view_rows

    @property
    def columns(self):
        return self._columns

    @property
    def rect(self) -> Rect:
        return self._view_rect

    @property
    def is_labeled(self) -> bool:
        return self._view_labeled

    @property
    def x_offset(self) -> int:
        return self._view_x_offset

    @property
    def y_offset(self) -> int:
        return self._view_y_offset

    def cell_width_px(self, cell: Cell) -> int:
        if cell.text.is_type(Text.COLUMN_SPAN):
            left = cell.column
            right = left + cell.text.span
            return sum([w for w in self._column_widths_px[left:right]])
        return self.column_width_px(cell.column)

    def column_width_px(self, column: int) -> int:
        """Get the width of a view column."""
        return self._column_widths_px[column] if 0 <= column < self.columns else 0

    def heading_height_px(self, row: int) -> int:
        """Get the height of a heading row."""
        return self._heading_heights_px[row] if 0 <= row < self._heading_rows else 0

    def content_height_px(self, row: int) -> int:
        """Get the height of a content row."""
        return self._view_heights[row] if 0 <= row < self.rows else 0

    def width_px(self) -> int:
        """Get the screen width of the view."""
        return self.x_offset + sum(self._column_widths_px[self._view_rect.left: self._view_rect.right])

    def height_px(self) -> int:
        """Get the screen height of the view."""
        return self.y_offset + sum(self._view_heights[self._view_rect.top: self._view_rect.bottom])

    def is_next_column_visible(self, width_px: int) -> bool:
        """Check to see if the next column is visible."""
        return (self.width_px() + 1 < width_px) if self._view_rect.right < self.columns else False

    def is_next_row_visible(self, height_px: int) -> bool:
        """Check to see if the next row is visible."""
        return (self.height_px() + 1 < height_px) if self._view_rect.bottom < self.rows else False

    def is_last_column_visible(self, width_px: int) -> bool:
        """Check to see if the last column in the view is visible."""
        view_width = sum(self._column_widths_px[self._view_rect.left: self._view_rect.right - 1])
        return (view_width + self.x_offset + 1) < width_px

    def is_last_row_visible(self, height_px: int) -> bool:
        """Check to see if the last row in the view is visible."""
        view_height = sum(h for h in self._view_heights[self._view_rect.top: self._view_rect.bottom - 1])
        return (self.y_offset + view_height + 1) < height_px
