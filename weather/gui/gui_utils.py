import errno
import os
import sys
from tkinter.font import Font, nametofont
from typing import Callable, NamedTuple, Tuple

from weather.configuration import get_logger

log = get_logger(__name__)

_short_month_names = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
_long_month_names = ["January", "February", "March", "April", "May", "June",
                     "July", "August", "September", "October", "November", "December"]


def month_name(month: int, zero_based: bool = False, short_names: bool = True):
    month_names = _short_month_names if short_names else _long_month_names
    return month_names[(month if zero_based else (month - 1)) % 12]


def make_font_builder(base_font: Font = None) -> Callable:
    if not base_font:
        base_font = nametofont("TkDefaultFont")

    def builder(size: int = None, weight: str = None, slant: str = None) -> Font:
        new_font = base_font.copy()
        kwargs = {}
        if size:
            kwargs["size"] = size
        if weight:
            kwargs["weight"] = weight
        if slant:
            kwargs["slant"] = slant
        if len(kwargs):
            new_font.config(**kwargs)
        return new_font

    return builder


def get_tag_builder(default_tag: str):
    def mk_tags(*tags: str) -> Tuple[str, ...]:
        return tuple([default_tag] + list(tags))

    return mk_tags


class Coord(NamedTuple):
    x: int
    y: int

    def __add__(self, other):
        return Coord(self.x + other.x, self.y + other.y)

    def with_x(self, x: int):
        return Coord(x, self.y)

    def with_x_offset(self, offset: int):
        return self.with_offset(offset, 0)

    def with_y_offset(self, offset: int):
        return self.with_offset(0, offset)

    def with_offset(self, x_offset, y_offset):
        return Coord(self.x + x_offset, self.y + y_offset)

    def __eq__(self, other):
        if isinstance(other, Coord):
            return self.x == other.x and self.y == other.y
        raise NotImplemented


################################################################################
# The is_pathname_valid code was pulled off stackoverflow (see https://stackoverflow.com/questions/9532499/
# check-whether-a-path-is-valid-in). There was no copyright and I don't want to
# claim this as my code. It simply worked and it's nice to have it around.
################################################################################

# Sadly, Python fails to provide the following magic number for us.
ERROR_INVALID_NAME = 123
'''
Windows-specific error code indicating an invalid pathname.

See Also
----------
https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes--0-499-
    Official listing of all such codes.
'''


def is_pathname_valid(pathname: str) -> bool:
    """
    `True` if the passed pathname is a valid pathname for the current OS;
    `False` otherwise.
    """
    # If this pathname is either not a string or is but is empty, this pathname
    # is invalid.
    try:
        if not isinstance(pathname, str) or not pathname:
            return False

        # Strip this pathname's Windows-specific drive specifier (e.g., `C:\`)
        # if any. Since Windows prohibits path components from containing `:`
        # characters, failing to strip this `:`-suffixed prefix would
        # erroneously invalidate all valid absolute Windows pathnames.
        _, pathname = os.path.splitdrive(pathname)

        # Directory guaranteed to exist. If the current OS is Windows, this is
        # the drive to which Windows was installed (e.g., the "%HOMEDRIVE%"
        # environment variable); else, the typical root directory.
        root_dirname = os.environ.get('HOMEDRIVE', 'C:') if sys.platform == 'win32' else os.path.sep
        assert os.path.isdir(root_dirname)  # ...Murphy and her ironclad Law

        # Append a path separator to this directory if needed.
        root_dirname = root_dirname.rstrip(os.path.sep) + os.path.sep

        # Test whether each path component split from this pathname is valid or
        # not, ignoring non-existent and non-readable path components.
        for pathname_part in pathname.split(os.path.sep):
            try:
                os.lstat(root_dirname + pathname_part)
            # If an OS-specific exception is raised, its error code
            # indicates whether this pathname is valid or not. Unless this
            # is the case, this exception implies an ignorable kernel or
            # filesystem complaint (e.g., path not found or inaccessible).
            #
            # Only the following exceptions indicate invalid pathnames:
            #
            # * Instances of the Windows-specific "WindowsError" class
            #   defining the "winerror" attribute whose value is
            #   "ERROR_INVALID_NAME". Under Windows, "winerror" is more
            #   fine-grained and hence useful than the generic "errno"
            #   attribute. When a too-long pathname is passed, for example,
            #   "errno" is "ENOENT" (i.e., no such file or directory) rather
            #   than "ENAMETOOLONG" (i.e., file name too long).
            # * Instances of the cross-platform "OSError" class defining the
            #   generic "errno" attribute whose value is either:
            #   * Under most POSIX-compatible OSes, "ENAMETOOLONG".
            #   * Under some edge-case OSes (e.g., SunOS, *BSD), "ERANGE".
            except OSError as exc:
                if hasattr(exc, 'winerror'):
                    if exc.winerror == ERROR_INVALID_NAME:
                        return False
                elif exc.errno in {errno.ENAMETOOLONG, errno.ERANGE}:
                    return False
    # If a "TypeError" exception was raised, it almost certainly has the
    # error message "embedded NUL character" indicating an invalid pathname.
    except TypeError as _:
        return False
    # If no exception was raised, all path components and hence this
    # pathname itself are valid. (Praise be to the curmudgeonly python.)
    else:
        return True
    # If any other exception was raised, this is an unrelated fatal issue
    # (e.g., a bug). Permit this exception to unwind the call stack.
    #
    # Did we mention this should be shipped with Python already?
