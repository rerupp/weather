from pathlib import Path
from sys import stderr
from tkinter import Menu, NSEW, PhotoImage, SE, SUNKEN, Tk
from tkinter.filedialog import askdirectory
from tkinter.ttk import Notebook, Sizegrip, Style
from typing import Callable, Dict

from py_weather_lib import PyLocation, PyWeatherConfig, create as create_rust_bindings

from .about import About
from .add_location import AddLocation
from .city_search import CitySearch
from .history_dates import HistoriesDates
from .history_summary import HistorySummary
from .infrastructure import WeatherEvent, WeatherView
from .locations import Locations
from ..config import get_logger
from ..domain import WeatherConfigException, WeatherData

__all__ = ['TkWeather']
log = get_logger(__name__)


class TkWeather:

    def __init__(self, weather_config: PyWeatherConfig):

        self._tk = Tk()

        # set up the main frame
        self._tk.title("Weather Data")
        self._tk.columnconfigure(0, weight=1)
        self._tk.rowconfigure(0, weight=1)

        # add the main menu
        self._tk.config(menu=_Menu(self._tk.event_generate))

        # initialize the components
        self._config = _Config(weather_config)
        self._notebook = _Notebook(self._tk)

        # set up the virtual events
        self._tk.bind(_MenuEvent.EXIT, lambda event: self._tk.quit())
        self._tk.bind(_MenuEvent.CHANGE_FOLDER, lambda event: self._change_folder())
        self._tk.bind(_MenuEvent.ADD_CUSTOM, lambda event: self._add_location())
        self._tk.bind(_MenuEvent.SEARCH_CITIES, lambda event: self._city_search_view())
        # self._tk.bind(_MenuEvent.PROPERTIES, lambda event: self._properties())
        # self._tk.bind(_MenuEvent.SETTINGS, lambda event: self._settings())
        self._tk.bind(_MenuEvent.VIEW_LOCATIONS, lambda event: self._locations_view())
        self._tk.bind(_MenuEvent.VIEW_HISTORIES, lambda event: self._histories_view())
        self._tk.bind(_MenuEvent.VIEW_SUMMARY, lambda event: self._summary_view())
        self._tk.bind(_MenuEvent.ABOUT, lambda event: self._about())
        self._tk.bind(WeatherEvent.REFRESH_VIEW, lambda event: self._notebook.refresh())

    def execute(self):
        try:
            self._config.init_bindings()
        except WeatherConfigException as e:
            print(str(e), file=stderr)
            exit(1)

        self._locations_view()

        # center (or try to...) the window on the screen
        window_width, window_height = self._notebook.winfo_width(), self._notebook.winfo_height()
        # the screen width is being reported as 2560x1440 on the 4k monitor
        screen_width, screen_height = self._tk.winfo_screenwidth(), self._tk.winfo_screenheight()
        x_position = int(screen_width / 3 - window_width / 2)
        y_position = int(screen_height / 3 - window_height / 2)
        self._tk.geometry("+{}+{}".format(x_position, y_position))
        min_width, min_height = self._notebook.size()
        self._tk.wm_minsize(min_width, min_height)
        self._tk.update()

        self._tk.mainloop()

    def _change_folder(self):
        folder = askdirectory(parent=self._notebook, title='Select the Weather Data folder', )
        if folder is not None and folder != '':
            self._config.set_directory(folder)
            self._notebook.refresh()

    def _add_location(self):
        if not AddLocation(self._tk, PyLocation(), self._config.as_weather_data()).is_cancelled():
            self._notebook.refresh()

    def _city_search_view(self):
        tab_name = 'City Search'
        if self._notebook.has_tab(tab_name):
            self._notebook.set_active(tab_name)
        else:
            search_cities = CitySearch(self._tk, self._config.as_weather_data())
            self._notebook.add_tab(tab_name, search_cities)

    # def _properties(self):
    #     print('properties')
    #
    # def _settings(self):
    #     print('settings')

    def _locations_view(self):
        tab_name = 'Locations'
        if self._notebook.has_tab(tab_name):
            self._notebook.set_active(tab_name)
        else:
            locations = Locations(self._notebook, self._config.as_weather_data(), self._notebook.add_tab)
            self._notebook.add_tab(tab_name, locations)

    def _histories_view(self):
        tab_name = 'History Dates'
        if self._notebook.has_tab(tab_name):
            self._notebook.set_active(tab_name)
        else:
            histories = HistoriesDates(self._notebook, self._config, self._notebook.add_tab)
            self._notebook.add_tab(tab_name, histories)

    def _summary_view(self):
        tab_name = 'History Summary'
        if self._notebook.has_tab(tab_name):
            self._notebook.set_active(tab_name)
        else:
            history_summary = HistorySummary(self._config.as_weather_data(), self._notebook)
            self._notebook.add_tab(tab_name, history_summary)

    def _about(self):
        About(self._tk)


class _Config(WeatherData):
    """
    The configuration front ends WeatherData bindings. This allows the weather data
    configuration to be changed at runtime without requiring all holders to be notified.
    """

    def __init__(self, weather_config: PyWeatherConfig):
        super().__init__()
        self._config = weather_config

    def set_directory(self, directory: str):
        if directory is None or directory == '':
            directory = 'weather_data'
        self._config.dirname = directory
        self.init_bindings()

    def as_weather_data(self) -> WeatherData:
        return self

    def init_bindings(self):
        dir_path = Path(self._config.dirname)
        if not dir_path.exists():
            raise WeatherConfigException('Weather data directory ({}) does not exist.'.format(dir_path))
        elif not dir_path.is_dir():
            raise WeatherConfigException('Weather data directory ({}) is not a directory.'.format(dir_path))
        try:
            # replace the managed Rust bindings in the super class
            self._backend = create_rust_bindings(self._config)
        except SystemExit as error:
            raise WeatherConfigException('Rust weather data bindings error.') from error


class _MenuEvent:
    CHANGE_FOLDER = '<<ChangeFolder>>'
    ADD_CUSTOM = '<<AddCustom>>'
    SEARCH_CITIES = '<<SearchCities>>'
    PROPERTIES = '<<Properties>>'
    SETTINGS = '<<FileSettings>>'
    EXIT = '<<WeatherExit>>'
    VIEW_LOCATIONS = '<<ViewLocations>>'
    VIEW_HISTORIES = '<<ViewHistories>>'
    VIEW_SUMMARY = '<<ViewSummary>>'
    ABOUT = '<<AboutWeather>>'


class _Menu(Menu):
    def __init__(self, generate_event: Callable[[str], None]):
        super().__init__(tearoff=False)

        # The toplevel file menu
        file = Menu(self, tearoff=0)
        self.add_cascade(label="File", underline=0, menu=file)

        # allow the weather data folder to be changed at runtime
        file.add_command(label='Change data folder', underline=0,
                         command=lambda: generate_event(_MenuEvent.CHANGE_FOLDER))

        # allow a new location to be created
        file.add_separator()
        file.add_command(label="Custom Location", underline=0, command=lambda: generate_event(_MenuEvent.ADD_CUSTOM))
        file.add_separator()

        # settings and properties
        file.add_command(label="Properties", underline=0, command=lambda: generate_event(_MenuEvent.PROPERTIES))
        file.add_command(label="Settings", underline=0, command=lambda: generate_event(_MenuEvent.SETTINGS))
        file.add_separator()
        file.add_command(label="Exit", underline=1, command=lambda: generate_event(_MenuEvent.EXIT))

        # search
        search = Menu(self, tearoff=0)
        self.add_cascade(label="Search", underline=0, menu=search)
        search.add_command(label="US Cities", underline=0, command=lambda: generate_event(_MenuEvent.SEARCH_CITIES))

        # The toplevel view menu
        view = Menu(self, tearoff=0)
        self.add_cascade(label="View", underline=0, menu=view)
        view.add_command(label="Locations", underline=0, command=lambda: generate_event(_MenuEvent.VIEW_LOCATIONS))
        view.add_command(label="Histories", underline=0, command=lambda: generate_event(_MenuEvent.VIEW_HISTORIES))
        view.add_command(label="Summary", underline=0, command=lambda: generate_event(_MenuEvent.VIEW_SUMMARY))

        #
        help_ = Menu(self, tearoff=0)
        help_.add_command(label="About...", underline=0, command=lambda: generate_event(_MenuEvent.ABOUT))
        self.add_cascade(label="Help", underline=0, menu=help_)


class _Views(Dict[str, WeatherView]):
    """
    This exists because it needs to be shared between the model and view.
    """

    def __init__(self):
        super().__init__()

    def add(self, tab_name: str, weather_view: WeatherView):
        log.debug(f'RefreshViews adding {tab_name}.')
        self[tab_name] = weather_view

    def remove(self, tab_name: str):
        if tab_name in self:
            log.debug(f'RefreshViews removing {tab_name}.')
            del self[tab_name]
        else:
            log.debug(f'RefreshViews {tab_name} not found...')

    def refresh(self):
        for view in self.keys():
            self[view].refresh()


__NOTEBOOK_INITIALIZED__ = False


class _Notebook(Notebook):
    """ A light wrapper around the notebook that's used as the main view. """

    def __init__(self, parent: Tk):

        # use an 'X' to close the notebook tab
        # there is only 1 weather notebook so don't worry if the style has already been
        # initialized (at least right now???)
        self.__initialize_notebook_style()

        super().__init__(parent, **{'style': 'WeatherNotebook'})
        parent.config(borderwidth=3, relief=SUNKEN)

        # set up the notebook area
        self.grid(sticky=NSEW)
        self.columnconfigure(0, weight=1)
        self.rowconfigure(0, weight=1)
        Sizegrip(parent).grid(row=1, sticky=SE)

        self.bind("<ButtonPress-1>", self._on_close_press, True)
        self.bind("<ButtonRelease-1>", self._on_close_release)
        self._active = None

        self._views: Dict[str, WeatherView] = {}

    def add_tab(self, tab_name: str, weather_view: WeatherView):
        """Add a tab to the notebook and make it the selected tab."""
        if tab_name in self._views:
            # the history and graph reports need to delete the older view
            log.debug('remove existing tab %s.', tab_name)
            for i in self.tabs():
                if self.tab(i, "text") == tab_name:
                    self.forget(i)
            del self._views[tab_name]
        log.debug(f'Adding tab {tab_name}.')
        self._views[tab_name] = weather_view
        self.add(weather_view.view(), text=tab_name)
        tabs = self.index('end')
        self.select(tabs - 1)

    def has_tab(self, tab_name: str) -> bool:
        """Query if the tab is in the notebook."""
        for i in self.tabs():
            if self.tab(i, "text") == tab_name:
                return True
        return False

    def set_active(self, tab_name: str):
        """Set a tab in the notebook active"""
        for i in self.tabs():
            if self.tab(i, "text") == tab_name:
                self.select(i)
                return
        log.error(f'Notebook set_active: {tab_name} tab not found.')

    def refresh(self):
        # Originally I tried using bind/unbind for views that were interested in refresh but there
        # is a bug in tkinter where unbind removes all bindings (been around 10 years).
        #
        # Bug Tracker old: #31485 new: #75666.
        #
        # #75666 had this as the fix but as of Jan-11-2024 the issue had not been closed
        #
        # def unbind(widget, seq, func_id):
        #     bindings = {x.split()[1][3:]: x for x in widget.bind(seq).splitlines() if x.strip()}
        #     try:
        #         del bindings[func_id]
        #     except KeyError:
        #         raise tk.TclError('Binding "%s" not defined.' % func_id)
        #     widget.bind(seq, '\n'.join(list(bindings.values())))
        for view in self._views.values():
            view.refresh()

    def _on_close_press(self, event):
        """Called when the button is pressed over the close button"""

        # check to see if the close image was pressed
        element = self.identify(event.x, event.y)
        if "close" in element:
            index = self.index("@%d,%d" % (event.x, event.y))
            self.state(['pressed'])
            self._active = index
            return "break"

    def _on_close_release(self, event):
        """Called when the button is released"""
        if not self.instate(['pressed']):
            return

        element = self.identify(event.x, event.y)
        if "close" not in element:
            # user moved the mouse off of the close button
            return

        # get the tab that was clicked
        index = self.index("@%d,%d" % (event.x, event.y))
        if self._active == index:
            tab_name = self.tab(index, "text")
            if tab_name in self._views:
                log.debug(f'Removing tab {tab_name}.')
                del self._views[tab_name]
            else:
                log.error(f'WeatherView {tab_name} not found...')
            self.forget(index)

        self.state(["!pressed"])
        self._active = None

        if self.index('end') == 0:
            self.quit()

    # noinspection SpellCheckingInspection
    def __initialize_notebook_style(self):
        # there's magic here, these image names show up when the notebook image_names() method is called
        self.images = (
            PhotoImage("img_close", data='''
                R0lGODlhCAAIAMIBAAAAADs7O4+Pj9nZ2Ts7Ozs7Ozs7Ozs7OyH+EUNyZWF0ZWQg
                d2l0aCBHSU1QACH5BAEKAAQALAAAAAAIAAgAAAMVGDBEA0qNJyGw7AmxmuaZhWEU
                5kEJADs=
                '''),
            PhotoImage("img_close_active", data='''
                R0lGODlhCAAIAMIEAAAAAP/SAP/bNNnZ2cbGxsbGxsbGxsbGxiH5BAEKAAQALAAA
                AAAIAAgAAAMVGDBEA0qNJyGw7AmxmuaZhWEU5kEJADs=
                '''),
            PhotoImage("img_close_pressed", data='''
                R0lGODlhCAAIAMIEAAAAAOUqKv9mZtnZ2Ts7Ozs7Ozs7Ozs7OyH+EUNyZWF0ZWQg
                d2l0aCBHSU1QACH5BAEKAAQALAAAAAAIAAgAAAMVGDBEA0qNJyGw7AmxmuaZhWEU
                5kEJADs=
            ''')
        )

        style = Style()
        style.element_create("close", "image", "img_close",
                             ("active", "pressed", "!disabled", "img_close_pressed"),
                             ("active", "!disabled", "img_close_active"), border=8, sticky='')
        style.layout("WeatherNotebook", [("WeatherNotebook.client", {"sticky": "nswe"})])
        style.layout("WeatherNotebook.Tab", [
            ("WeatherNotebook.tab", {
                "sticky": "nswe",
                "children": [
                    ("WeatherNotebook.padding", {
                        "side": "top",
                        "sticky": "nswe",
                        "children": [
                            ("WeatherNotebook.focus", {
                                "side": "top",
                                "sticky": NSEW,
                                "children": [
                                    ("WeatherNotebook.label", {"side": "left", "sticky": ''}),
                                    ("WeatherNotebook.close", {"side": "left", "sticky": ''}),
                                ]
                            })
                        ]
                    })
                ]
            })
        ])
