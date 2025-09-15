__all__ = ['Rect']


class Rect:
    def __init__(self, left=0, top=0, width=0, height=0):
        self._left = left
        self._top = top
        self._width = width
        self._height = height

    def __str__(self):
        return f'{self.__class__.__name__}({self._width}x{self._height}+{self._left}+{self._top})'

    @property
    def left(self):
        """The left position of the area."""
        return self._left

    @left.setter
    def left(self, value):
        """Set the left position of the area."""
        if value < 0:
            raise ValueError('left must be positive')
        self._left = value

    @property
    def top(self):
        """The top position of the area."""
        return self._top

    @top.setter
    def top(self, value):
        """Set the top position of the area."""
        if value < 0:
            raise ValueError('top must be positive')
        self._top = value

    @property
    def width(self):
        """The width of the area."""
        return self._width

    @width.setter
    def width(self, value):
        """Set the width of the area."""
        if value < 0:
            raise ValueError('width must be positive')
        self._width = value

    @property
    def height(self):
        """The height of the area."""
        return self._height

    @height.setter
    def height(self, value):
        """Set the height of the area."""
        if value < 0:
            raise ValueError('height must be positive')
        self._height = value

    @property
    def right(self):
        """The position just right of the area."""
        return self._left + self._width

    @property
    def bottom(self):
        """The position just below the area."""
        return self._top + self._height

    def is_empty(self) -> bool:
        """Return True if width and height is 0."""
        return self._width == 0 and self._height == 0
