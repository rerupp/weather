from typing import Tuple
from dataclasses import dataclass

__all__ = ['Color', 'DefaultColors']


class Color:
    def __init__(self, red: int, green: int, blue: int):
        if red < 0 or red > 2**16:
            raise ValueError("red must be an unsigned 16-bit integer")
        if green < 0 or green > 2**16:
            raise ValueError("green must be an unsigned 16-bit integer")
        if blue < 0 or blue > 2**16:
            raise ValueError("blue must be an unsigned 16-bit integer")
        self._red = red
        self._green = green
        self._blue = blue
        self._tkinter_rgb = f'#{self._red:04x}{self._green:04x}{self._blue:04x}'

    def __str__(self):
        return self._tkinter_rgb

    @property
    def rgb(self) -> Tuple[int, int, int]:
        return self._red, self._green, self._blue


@dataclass(frozen=True)
class DefaultColors:
    # the rgb for Windoz SystemButtonFace
    background=Color(61680, 61680, 61680)
    # black
    outline=Color(0, 0, 0)
