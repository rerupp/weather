import tkinter as tk
from typing import Callable, List, Tuple

from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
from matplotlib.figure import Figure

__all__ = ['GraphView']


class GraphView(tk.Frame):
    """Isolate the matplotlib Tk backend to this class."""

    def __init__(self, parent, figure: Figure):
        super().__init__(parent)
        self.columnconfigure(0, weight=1)
        self.rowconfigure(0, weight=1)

        # draw the figure
        self._canvas = FigureCanvasTkAgg(figure, master=self)
        self._canvas.draw()

        # show the figure
        self._canvas.get_tk_widget().grid(sticky="nsew")

        # remember the event handlers
        self._event_handlers: List[Tuple[str, Callable[[tk.Event], None]]] = []

    def add_handler(self, event: str, handler: Callable[[tk.Event], None]):
        """Bind an event handler to the Tk canvas."""
        self._canvas.get_tk_widget().bind(event, handler)
        self._event_handlers.append((event, handler))

    def replace(self, figure: Figure, clear_events=False):
        self._canvas.get_tk_widget().destroy()
        self._canvas = FigureCanvasTkAgg(figure, master=self)
        if clear_events:
            self._event_handlers.clear()
        for (event, handler) in self._event_handlers:
            # self.add_handler(event, handler)
            self._canvas.get_tk_widget().bind(event, handler)
