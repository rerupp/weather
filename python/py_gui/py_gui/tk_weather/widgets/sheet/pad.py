__all__ = ['Pad']


class Pad:
    def __init__(self, left=0, top=0, right=0, bottom=0, surround=0):
        if surround > 0:
            left = top = right = bottom = surround
        self._left = left
        self._top = top
        self._right = right
        self._bottom = bottom

    def __str__(self):
        return f'{self.__class__.__name__}({self._left},{self._top},{self._right},{self._bottom}]'

    @property
    def left(self):
        return self._left

    @property
    def top(self):
        return self._top

    @property
    def right(self):
        return self._right

    @property
    def bottom(self):
        return self._bottom
