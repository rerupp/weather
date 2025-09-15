######################################################################################
#
# GUI colors
#
######################################################################################
from typing import NamedTuple, Tuple

import importlib_resources

from .. import get_logger


class Color(NamedTuple):
    name: str
    red: int
    green: int
    blue: int

    def to_hex(self):
        return "#{:02x}{:02x}{:02x}".format(self.red, self.green, self.blue)


RGB_TXT_NAME = 'rgb.txt'


def load_colors() -> Tuple[Color, ...]:
    content = []
    try:
        colors = importlib_resources.files(__package__).joinpath(RGB_TXT_NAME).read_text()
        for color in colors.splitlines():
            if not color.startswith("!"):
                red, green, blue, name = color.split(maxsplit=3)
                content.append(Color(name, int(red), int(green), int(blue)))
    except ImportError as err:
        get_logger(__name__).error("Yikes... {}!".format(err.msg))
    except FileNotFoundError as err:
        get_logger(__name__).error("Yikes... {} '{}'!".format(err.strerror, err.filename))

    # make sure colors are returned
    if not content:
        get_logger(__name__).warning("Creating colors tkinter guarantees")
        content = [
            Color("white", 255, 255, 255),
            Color("black", 0, 0, 0),
            Color("red", 255, 0, 0),
            Color("green", 0, 255, 0),
            Color("blue", 0, 0, 255),
            Color("cyan", 0, 255, 255),
            Color("yellow", 255, 255, 0),
            Color("magenta", 255, 0, 255)
        ]
    return tuple(content)
