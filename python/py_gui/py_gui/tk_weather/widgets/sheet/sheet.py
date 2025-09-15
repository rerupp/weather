from tkinter import (Canvas, Frame, MOVETO, PAGES, SCROLL, NSEW, N, S, E, W, Scrollbar)
from typing import Iterable

from .data import Data
from .renderer import Renderer
from .row import Row
from .scroller import Scroller
from ...infrastructure import Stopwatch
from ....config import get_logger

__all__ = ['Sheet']
log = get_logger(__name__)


class Sheet(Frame):
    def __init__(self, parent, headings: Iterable[Row], contents: Iterable[Row], view_labeled: bool):
        # initialize the main frame
        super().__init__(parent)
        self.columnconfigure(0, weight=1)
        self.rowconfigure(0, weight=1)
        self.grid(row=0, column=0, sticky=NSEW)

        self._canvas = Canvas(self, borderwidth=0, highlightthickness=0)
        self._canvas.focus_set()
        self._canvas.bind('<Configure>', self._canvas_config_handler)

        # bootstrap the canvas managers
        self._data = Data(headings, contents, view_labeled)
        self._renderer = Renderer(self._data)
        self._scroller = Scroller(self._data, self._renderer)

        # add in the scrollbars
        self._v_scrollbar = Scrollbar(self, orient="vertical", command=self._v_scrollbar_handler)
        self._h_scrollbar = Scrollbar(self, orient="horizontal", command=self._h_scrollbar_handler)

        # position the view
        self._canvas.grid(column=0, row=0, sticky=NSEW)
        self._v_scrollbar.grid(row=0, column=1, sticky=N + S + E, rowspan=2)
        self._h_scrollbar.grid(row=1, column=0, sticky=W + S + E)

        # add the event bindings
        self._canvas.bind('<MouseWheel>', self._mouse_wheel_handler)
        self._canvas.bind('<Left>', lambda _: self._h_scroll(SCROLL, -1))
        self._canvas.bind('<Right>', lambda _: self._h_scroll(SCROLL, 1))
        self._canvas.bind('<Up>', lambda _: self._v_scroll(SCROLL, -1))
        self._canvas.bind('<Down>', lambda _: self._v_scroll(SCROLL, 1))
        self._canvas.bind('<Prior>', lambda _: self._v_scroll(PAGES, -1))
        self._canvas.bind('<Next>', lambda _: self._v_scroll(PAGES, 1))
        self._canvas.bind('<F5>', lambda _: self.refresh_view())

    def refresh_view(self):
        """Redraw the current view and update the scrollbars."""
        stopwatch = Stopwatch()
        self._canvas.delete('all')
        self._renderer.refresh_view(self._canvas)
        self._update_scrollbars()
        log.info('refresh_view %s', stopwatch)

    def _canvas_config_handler(self, _):
        """Update the view when a configure event is received."""
        stopwatch = Stopwatch()
        bbox = self._canvas.bbox('all')
        self._canvas.configure(scrollregion=bbox)
        if self._data.view.rect.is_empty():
            self._initialize_view()
            return

        # refresh the width
        window_width_px = self._window_width_px()
        view_width_px = self._data.view.width_px()
        if window_width_px < view_width_px:
            self._renderer.trim_columns(self._canvas, window_width_px)
        elif window_width_px > view_width_px:
            self._renderer.append_columns(self._canvas, window_width_px)

        # refresh the height
        window_height_px = self._window_height_px()
        view_height_px = self._data.view.height_px()
        if window_height_px < view_height_px:
            self._renderer.trim_rows(self._canvas, window_height_px)
        elif window_height_px > view_height_px:
            self._renderer.append_rows(self._canvas, window_height_px)

        # make sure the scrollbars are up to date
        self._update_scrollbars()
        log.info('_canvas_config_handler %s', stopwatch)

    def _v_scrollbar_handler(self, *args):
        """Update the view when the vertical scrollbar is changed."""
        if args[0] == MOVETO:
            self._v_scroll(MOVETO, round(float(self._data.view.rows) * min(max(float(args[1]), 0.0), 1.0)))
        elif args[2] == PAGES:
            self._v_scroll(PAGES, int(args[1]))
        else:
            self._v_scroll(SCROLL, int(args[1]))

    def _h_scrollbar_handler(self, *args):
        """Update the view when the horizontal scrollbar is changed."""
        if args[0] == MOVETO:
            self._h_scroll(MOVETO, round(float(self._data.view.columns) * min(max(float(args[1]), 0.0), 1.0)))
        elif args[2] == PAGES:
            self._h_scroll(PAGES, int(args[1]))
        else:
            self._h_scroll(SCROLL, int(args[1]))

    def _mouse_wheel_handler(self, event):
        """Update the view when a mouse wheel is received."""
        # on Linux Button-4 is up, Button-5 is down, and on windows the delta will be a multiple of +-120
        direction = 1 if event.num == 4 else -1 if event.num == 5 else (int(event.delta / 120))
        # 8 is the Left-hand Alt modifier
        if event.state == 8:
            self._v_scroll(SCROLL, direction)
        # 9 is the Left-hand Alt | Shift modifiers
        elif event.state == 9:
            self._h_scroll(SCROLL, direction)
        else:
            log.error('Unsupported event state %s', event)

    def _window_height_px(self) -> int:
        """The height of the canvas in pixels."""
        return self._canvas.winfo_height()

    def _window_width_px(self) -> int:
        """The width of the canvas in pixels."""
        return self._canvas.winfo_width()

    def _update_scrollbars(self):
        """Updates the horizontal and vertical scrollbars."""
        # don't get clobbered by empty contents
        if not self._data.contents:
            return
        view = self._data.view
        # vertical
        v_first = float(view.rect.top) / float(view.rows)
        v_last = float(view.rect.bottom) / float(view.rows)
        self._v_scrollbar.set(v_first, min(v_last, 1.0))
        # horizontal
        h_first = float(view.rect.left) / float(view.columns)
        h_last = float(view.rect.right) / float(view.columns)
        self._h_scrollbar.set(h_first, min(h_last, 1.0))

    def _initialize_view(self):
        """Update the view to reflect the current canvas size."""
        stopwatch = Stopwatch()
        # get the number of columns that will fit on the screen
        view = self._data.view
        window_width = self._window_width_px()
        while view.is_next_column_visible(window_width):
            view.rect.width += 1
        view.rect.width = max(1, view.rect.width)

        # get the number of rows that will fit on the screen
        window_height = self._window_height_px()
        while view.is_next_row_visible(window_height):
            view.rect.height += 1
        view.rect.height = max(1, view.rect.height)

        self.refresh_view()
        log.info('_initialize_view %s', stopwatch)

    def _v_scroll(self, mode: str, scroll_amount: int):
        """Update the view to reflect a vertical scroll event."""
        stopwatch = Stopwatch()
        scrolled = False
        if mode == PAGES:
            if scroll_amount > 0:
                scrolled = self._scroller.page_down(self._canvas, self._window_height_px())
            elif scroll_amount < 0:
                scrolled = self._scroller.page_up(self._canvas)
            else:
                log.warning('_v_scroll %s direction is 0.', mode)
        elif mode == SCROLL:
            if scroll_amount > 0:
                scrolled = self._scroller.scroll_down(self._canvas, self._window_height_px())
            elif scroll_amount < 0:
                scrolled = self._scroller.scroll_up(self._canvas)
            else:
                log.warning('_v_scroll %s direction is 0.', mode)
        elif mode == MOVETO:
            scrolled = self._scroller.moveto_row(self._canvas, scroll_amount)
        else:
            log.warning('_v_scroll mode (%s) not supported', mode)
        if scrolled:
            self._update_scrollbars()
        log.info('_v_scroll %s', stopwatch)

    def _h_scroll(self, mode: str, scroll_amount: int):
        """Update the view to reflect a horizontal scroll event."""
        stopwatch = Stopwatch()
        scrolled = False
        if mode == PAGES:
            if scroll_amount > 0:
                scrolled = self._scroller.page_right(self._canvas, self._window_width_px())
            elif scroll_amount < 0:
                scrolled = self._scroller.page_left(self._canvas)
            else:
                log.warning('_h_scroll direction is 0.')
        elif mode == SCROLL:
            if scroll_amount > 0:
                scrolled = self._scroller.scroll_right(self._canvas, self._window_width_px())
            elif scroll_amount < 0:
                scrolled = self._scroller.scroll_left(self._canvas, self._window_width_px())
            else:
                log.warning('_h_scroll direction is 0.')
        elif mode == MOVETO:
            scrolled = self._scroller.moveto_column(self._canvas, scroll_amount, self._window_width_px())
        else:
            log.warning(f'_h_scroll mode (%s) not supported', mode)
        if scrolled:
            self._update_scrollbars()
        log.info('_h_scroll %s', stopwatch)
