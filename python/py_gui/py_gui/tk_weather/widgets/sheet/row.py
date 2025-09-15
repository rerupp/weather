from tkinter import LEFT
from tkinter.font import nametofont
from typing import List, Iterable

from .cell import (Cell, CellStyle)
from .pad import Pad
from .text import Text

__all__ = ['Column', 'Row']


class Column:
    __span_pad = Pad()
    # __span_style = CellStyle(nametofont('TkDefaultFont'))
    __span_style: CellStyle = None

    def __init__(self, text: str, pad: Pad, style: CellStyle, **kwargs):
        self._text = text
        self._pad = pad
        self._style = style
        self._justify = kwargs['justify'] if 'justify' in kwargs else LEFT
        self._header = kwargs['header'] if 'header' in kwargs else False
        self._content = kwargs['content'] if 'content' in kwargs else False
        self._label = kwargs['label'] if 'label' in kwargs else False
        self._span = kwargs['span'] if 'span' in kwargs else 1
        self._row_span = kwargs['row_span'] if 'row_span' in kwargs else False
        self._column_span = kwargs['column_span'] if 'column_span' in kwargs else False

    @staticmethod
    def header(text: str, pad: Pad, style: CellStyle, justify=LEFT, label=False, row_span=1, column_span=1):
        row_span = max(1, row_span)
        column_span = max(1, column_span)
        if row_span > 1 and column_span > 1:
            raise ValueError('Using both row_span and column_span is not supported.')
        span = row_span if row_span > 1 else column_span
        return Column(text, pad, style, justify=justify, header=True, label=label, span=span,
                      row_span=True if row_span > 1 else False, column_span=True if column_span > 1 else False)

    @staticmethod
    def content(text: str, pad: Pad, style: CellStyle, justify=LEFT, label=False) -> 'Column':
        return Column(text, pad, style, justify=justify, content=True, label=label, )

    @staticmethod
    def span():
        return Column('', Column.__span_pad, Column.span_style(), span=0)

    @staticmethod
    def span_style() -> CellStyle:
        if Column.__span_style is None:
            Column.__span_style = CellStyle(nametofont('TkDefaultFont'))
        return Column.__span_style

    def to_cell(self, col, row) -> Cell:
        text = Text(self._text, justify=self._justify, header=self._header, content=self._content, label=self._label,
                    span=self._span, row_span=self._row_span, column_span=self._column_span)
        return Cell(col, row, text, self._pad, self._style)


class Row:
    def __init__(self, row: int, columns: Iterable[Column], is_header=False):
        # create the cells and get the line metrics
        self._columns: List[Cell] = [column.to_cell(column_index, row) for column_index, column in enumerate(columns)]
        self._index = row
        self._height_px = max([cell.height for cell in self._columns])
        if not self._height_px:
            # the row is empty or the columns are all spans
            self._height_px = Column.span_style().font.metrics('linespace')
        self._len = len(self._columns)
        self._is_header = is_header

    @property
    def is_header(self):
        return self._is_header

    @property
    def index(self) -> int:
        return self._index

    @property
    def height_px(self) -> int:
        return self._height_px

    def __str__(self):
        return f'Row({self._index})'

    def __getitem__(self, item):
        """Return either a cell or slice of the row."""
        return self._columns[item]

    def __iter__(self):
        """Return an iterator over the row."""
        return self._columns.__iter__()

    def __len__(self) -> int:
        """Return the number of columns in the row."""
        return self._len
