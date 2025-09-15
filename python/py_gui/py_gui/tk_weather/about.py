from tkinter import ACTIVE, Button, E, Frame, LEFT, Label, W
from tkinter.simpledialog import Dialog

import py_weather_lib as wd

from ..metadata import __name__, __version__

__all__ = ['About']


class About(Dialog):
    def __init__(self, parent):
        super().__init__(parent, title=f'About {__name__}')

    def body(self, master):
        (Label(master, text="A Tk GUI using pyo3 bindings to call\nthe weather data Rust backend.")
         .grid(row=0, columnspan=2, pady=5))
        Label(master, text=f'{__name__}:').grid(row=1, column=0, sticky=E, pady=0)
        Label(master, text=__version__).grid(row=1, column=1, sticky=W, pady=0)
        Label(master, text=f'{wd.__name__}:').grid(row=2, column=0, sticky=E, pady=0)
        Label(master, text=wd.__version__).grid(row=2, column=1, sticky=W, pady=0)

    def buttonbox(self):
        box = Frame(self, padx=5, pady=5)
        w = Button(box, text="Ok", width=10, command=self.cancel, default=ACTIVE)
        w.pack(side=LEFT, padx=5, pady=5)
        self.bind("<Escape>", self.cancel)
        box.pack()
