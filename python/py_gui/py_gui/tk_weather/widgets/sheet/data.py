from collections.abc import Iterable
from itertools import chain

from .row import Row
from .text import Text
from .view import View

__all__ = ['Data']


class Data:
    """The Details class is used internally and holds information for a sheet."""
    def __init__(self, headings: Iterable[Row], contents: Iterable[Row], view_labeled=True):
        # save the headings and contents
        self.headings = [heading for heading in headings]
        self.contents = [content for content in contents]

        # get the maximum number of columns
        self._column_count = max(len(row) for row in chain(self.headings, self.contents))

        # get the max column widths
        column_widths = [0] * self._column_count
        for row in chain(self.headings, self.contents):
            for cell in row:
                if not cell.text.is_type(Text.COLUMN_SPAN):
                    column_widths[cell.column] = max(column_widths[cell.column], cell.width)

        # create the view
        heading_heights = [row.height_px for row in self.headings]
        content_heights = [row.height_px for row in self.contents]
        self.view = View(column_widths, heading_heights, content_heights, view_labeled)
