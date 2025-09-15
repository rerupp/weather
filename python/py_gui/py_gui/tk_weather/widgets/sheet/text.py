from tkinter import (CENTER, LEFT, RIGHT)
from typing import Union

__all__ = ['Text']


class Text:
    HEADER = 0x01
    CONTENT = 0x02
    LABEL = 0x04
    # CELL_SPAN = 0x08
    ROW_SPAN = 0x10
    COLUMN_SPAN = 0x20

    def __init__(self, value: str, justify=LEFT, header=False, content=False, label=False, span=1, row_span=False,
                 column_span=False):
        if justify not in (CENTER, LEFT, RIGHT):
            raise ValueError("Justify must be one of CENTER, LEFT, or RIGHT")
        if span > 1 and row_span is False and column_span is False:
            raise ValueError("Either row_span or column_span needs to be set")
        if row_span and column_span:
            raise ValueError("Having both row_span or column_span set is not supported")
        self._value = '' if value is None else str(value)
        self._justify = justify
        self._span = max(0, span)
        self._flags = Text.HEADER if header else Text.CONTENT if content else 0
        if label:
            self._flags |= Text.LABEL
        if row_span:
            self._flags |= Text.ROW_SPAN
        if column_span:
            self._flags |= Text.COLUMN_SPAN

    @staticmethod
    def header(value: str, justify=LEFT, label=False, span=1, row_span=False, column_span=False) -> 'Text':
        return Text(value, justify, header=True, label=label, span=span, row_span=row_span, column_span=column_span)

    @staticmethod
    def content(value: str, justify=LEFT, label=False, span=1, row_span=False, column_span=False) -> 'Text':
        return Text(value, justify, content=True, label=label, span=span, row_span=row_span, column_span=column_span)

    def __str__(self) -> str:
        flags = []
        if self.is_type(Text.HEADER):
            flags.append('HEADER')
        if self.is_type(Text.CONTENT):
            flags.append('CONTENT')
        if self.is_type(Text.LABEL):
            flags.append('LABEL')
        if self.is_type(Text.ROW_SPAN):
            flags.append('ROW_SPAN')
        if self.is_type(Text.COLUMN_SPAN):
            flags.append('COLUMN_SPAN')
        value = self._value.replace('\n', '\\n') if len(self._value) else ''
        return f'{self.__class__.__name__}("{value}",span={self._span},flags={self._flags:08b} [{", ".join(flags)}])'

    @property
    def value(self) -> str:
        return self._value

    @property
    def justify(self) -> Union[CENTER, LEFT, RIGHT]:
        return self._justify

    @property
    def span(self) -> int:
        return self._span

    def is_span(self) -> bool:
        return not (self.is_type(Text.HEADER) or self.is_type(Text.CONTENT))

    def is_type(self, flag: int) -> bool:
        return self._flags & flag == flag
