from tkinter import (Canvas, N, NE, NW, CENTER, LEFT, RIGHT, W, E)
from tkinter.font import Font
from typing import Iterable

from .color import (Color, DefaultColors)
from .pad import Pad
from .text import Text
from ....config import get_logger

__all__ = ['Cell', 'CellStyle']
log = get_logger(__name__)


class CellStyle:
    def __init__(self, font: Font, background=DefaultColors.background, outlined=False,
                 outline_color=DefaultColors.outline):
        self._font = font
        self._background = background
        self._outlined = outlined
        self._outline_color = outline_color

    @property
    def font(self) -> Font:
        return self._font

    @property
    def background(self) -> Color:
        return self._background

    @property
    def outlined(self) -> bool:
        return self._outlined

    @property
    def outline_color(self) -> Color:
        return self._outline_color


class Cell:
    def __init__(self, col: int, row: int, text: Text, pad: Pad, style: CellStyle):
        self._col = col
        self._row = row
        self._text = text
        self._pad = pad
        self._style = style
        # if text.is_type(Text.ROW_SPAN) or text.is_type(Text.COLUMN_SPAN):
        if text.is_span():
            self._text_width = self._width = self._height = 0
        else:
            # self._text_width = style.font.measure(text.value)
            size = style.font.measure
            self._text_width = max([size(t) for t in text.value.splitlines()]) if len(text.value) else size('')
            self._width = pad.left + self._text_width + pad.right + (2 if style.outlined else 0)
            self._height = pad.top + style.font.metrics('linespace') + pad.bottom + (2 if style.outlined else 0)

    def __str__(self):
        return f'{self.__class__.__name__}({self._col},{self._row},{self._text})'

    @property
    def row(self) -> int:
        return self._row

    @property
    def column(self) -> int:
        return self._col

    @property
    def width(self) -> int:
        return self._width

    @property
    def height(self) -> int:
        return self._height

    @property
    def text(self) -> Text:
        return self._text

    def at(self, col: int, row: int) -> 'Cell':
        """Create a copy of the cell with the given column and row."""
        return Cell(col, row, self._text, self._pad, self._style)

    def render(self, canvas: Canvas, x: int, y: int, width: int, height: int, tags: Iterable[str]):
        # don't render if this is a span
        if self._text.is_span():
            return
        style = self._style
        # create the cell background
        fill = str(style.background)
        outline = 1 if style.outlined else 0
        outlined_color = str(style.outline_color)
        tags = tuple(tags)
        canvas.create_rectangle(x, y, x + width, y + height, fill=fill, tags=tags, outline=outlined_color,
                                width=outline)
        # now adjust where the text will be placed
        if outline > 0:
            x += 1
            y += 1
            width -= 2
        x += self._pad.left
        y += self._pad.top
        width -= self._pad.left + self._pad.right
        anchor = NW
        if self._text.is_type(Text.ROW_SPAN):
            y += round(float(height) / 2.0)
            if self._text.justify == LEFT:
                anchor = W
            elif self._text.justify == CENTER:
                anchor = CENTER
                x += round(float(width) / 2.0)
            else:
                anchor = E
        elif self._text.justify == RIGHT:
            anchor = NE
            x += width
        elif self._text.justify == CENTER:
            anchor = N
            x += round(float(width) / 2.0)
        canvas.create_text(x, y, text=self._text.value, font=style.font, anchor=anchor, width=self._text_width,
                           tags=tags, justify=self._text.justify)
