from tkinter import Canvas

from .data import Data
from .renderer import Renderer
from ....config import get_logger

__all__ = ['Scroller']
log = get_logger(__name__)


class Scroller:
    def __init__(self, data: Data, renderer: Renderer):
        self._data = data
        self._renderer = renderer
        self._clear = lambda c: c.delete('all')

    def page_right(self, canvas: Canvas, width_px: int) -> bool:
        """Move the view one page to the right."""
        view = self._data.view
        paged = False
        # don't page past the right hand side
        if view.rect.right >= view.columns:
            log.debug('page_right already at right side')
        else:
            # adjust the lhs
            view.rect.left += (view.rect.width - 2)
            # make sure not to leave empty columns on the right hand side
            if view.rect.right >= view.columns:
                # don't mess with the width just get the lhs that shows the last column
                view.rect.left = view.columns - 1
                for column in reversed(range(view.columns - 1)):
                    if view.column_width_px(column) + view.width_px() > width_px:
                        break
                    view.rect.left -= 1
            # update the view
            self._clear(canvas)
            self._renderer.refresh_view(canvas)
            paged = True
        return paged

    def page_left(self, canvas: Canvas) -> bool:
        """Move the view one page to the left."""
        view = self._data.view
        paged = False
        # don't page past the content left hand side
        lhs_bound = 1 if view.is_labeled else 0
        if view.rect.left <= lhs_bound:
            pass
            log.debug('page_left already at left side')
        else:
            # make sure left hand side doesn't go out of bounds
            view.rect.left = max(lhs_bound, view.rect.left - view.rect.width - 1)
            # update the view
            self._clear(canvas)
            self._renderer.refresh_view(canvas)
            paged = True
        return paged

    def scroll_right(self, canvas: Canvas, width_px: int) -> bool:
        """Move the view one column to the right."""
        view = self._data.view
        # remember if you're already at the rhs
        at_rhs = view.rect.right >= view.columns
        # don't scroll past the rhs
        if at_rhs:
            # make sure the last column is completely visible
            if view.width_px() < width_px:
                log.debug('scroll_columns at right side')
                return False
        # delete the left column
        left = view.rect.left
        tag = view.col_tag(left)
        log.debug('scroll_columns deleting %s', tag)
        canvas.delete(tag)
        # scroll the view left
        scroll_amount = view.column_width_px(left)
        canvas.move(view.hscroll_tag, -scroll_amount, 0)
        # adjust the lhs
        view.rect.left += 1
        view.rect.width -= 1
        if self._renderer.is_header_left_top_span():
            self._renderer.render_header_left_top_span(canvas)
        if not at_rhs:
            self._renderer.append_columns(canvas, width_px)
        return True

    def scroll_left(self, canvas: Canvas, width_px: int) -> bool:
        """Move the view one column to the left."""
        view = self._data.view
        # don't scroll past the content lhs
        lhs_bound = 1 if view.is_labeled else 0
        if view.rect.left <= lhs_bound:
            log.debug('scroll_columns at left side')
            return False
        # adjust the view position and size
        view.rect.left -= 1
        view.rect.width += 1
        # scroll the view right
        x_scroll = view.column_width_px(view.rect.left)
        canvas.move(view.hscroll_tag, x_scroll, 0)
        # add the new lhs column
        self._renderer.add_column(canvas, view.rect.left, view.x_offset)
        # clean up any rhs columns that are not visible
        self._renderer.trim_columns(canvas, width_px)
        return True

    def moveto_column(self, canvas: Canvas, column: int, width_px: int) -> bool:
        """Scroll the view left or right depending on the column and view left side."""
        view = self._data.view
        # sanitize the column
        lhs = 1 if view.is_labeled else 0
        column = min(max(column, lhs), view.columns - view.rect.width)
        # bail if you get the same column
        if column == view.rect.left:
            return False
        log.debug('moveto_column=%s rect=%s', column, view.rect)
        if view.rect.left < column:
            return self.scroll_right(canvas, width_px)
        return self.scroll_left(canvas, width_px)

    def page_down(self, canvas: Canvas, height_px: int) -> bool:
        """Move the contents view down one page."""
        view = self._data.view
        # don't page past the bottom
        if view.rect.bottom >= view.rows:
            # make sure the last row is visible
            if height_px < view.height_px():
                return self.scroll_down(canvas, height_px)
            log.debug('page_down already at bottom')
            return False
        # page down the height minus 1 row
        view.rect.top += view.rect.height - 1
        # make sure not to leave empty columns on the right hand side
        if view.rect.bottom >= view.rows:
            view.rect.top = view.rows - view.rect.height
        self._clear(canvas)
        self._renderer.refresh_view(canvas)
        return True

    def page_up(self, canvas: Canvas) -> bool:
        """Move the contents view up one page."""
        view = self._data.view
        # don't page past the top
        if view.rect.top < 1:
            log.debug('_page_up already at top')
            return False
        # make sure top doesn't go out of bounds
        view.rect.top = max(0, view.rect.top - view.rect.height + 1)
        self._clear(canvas)
        self._renderer.refresh_view(canvas)
        return True

    def scroll_down(self, canvas: Canvas, height_px: int) -> bool:
        """Move the view down one row."""
        view = self._data.view
        # don't scroll past the bottom row
        at_bottom = view.rect.bottom >= view.rows
        if at_bottom:
            # make sure the last row is visible
            if height_px >= view.height_px():
                log.debug(f'scroll_row already at bottom row')
                return False
        # Delete the top row
        tag = view.row_tag(view.rect.top)
        log.debug('scroll_down deleting %s',tag)
        canvas.delete(tag)
        # scroll the view up
        scroll_amount = view.content_height_px(view.rect.top)
        canvas.move(view.vscroll_tag, 0, -scroll_amount)
        # add the row to the bottom of the view
        if not at_bottom:
            y_offset = view.height_px() - scroll_amount
            self._renderer.render_row(canvas, self._data.contents[view.rect.bottom], y_offset)
        # move the top to the next row
        view.rect.top += 1
        return True

    def scroll_up(self, canvas: Canvas) -> bool:
        """Move the view up one column."""
        view = self._data.view
        # don't scroll past the top
        if view.rect.top < 1:
            log.debug('_scroll_up already at top')
            return False
        # delete the bottom row and scroll the content down 1 row
        last_row = view.rect.bottom - 1
        tag = view.row_tag(last_row)
        log.debug('_scroll_up deleting %s', tag)
        canvas.delete(tag)
        canvas.move(view.vscroll_tag, 0, view.content_height_px(last_row))
        # the view top needs to be set to the previous row
        view.rect.top -= 1
        # add in the new top row
        self._renderer.render_row(canvas, self._data.contents[view.rect.top], view.y_offset)
        return True

    def moveto_row(self, canvas: Canvas, row: int) -> bool:
        """Set the top row of the view."""
        # force the lower bounds
        row = max(0, row)
        view = self._data.view
        log.debug('moveto_row=%s %s', row, view)
        # don't do anything unless there's a change
        if view.rect.top == row:
            log.debug('already at the top row')
            return False
        # check to see if you should be at the top row
        if row < 1:
            if view.rect.top == 0:
                log.debug('moveto_row already at top')
                return False
            view.rect.top = 0
        # check to see if you should be at the bottom row
        elif view.rows - view.rect.height < row:
            if view.rect.bottom >= view.rows:
                log.debug('moveto_row already at bottom')
                return False
            view.rect.top = view.rows - view.rect.height
        else:
            view.rect.top = row
        # refresh usually takes ~8-10 ms so that isn't a big source of slow
        self._clear(canvas)
        self._renderer.refresh_view(canvas)
        return True
