import tkinter as tk
from tkinter import messagebox
from tkinter.simpledialog import Dialog

from py_weather_lib import PyLocation, PyLocationFilter, PyLocationFilters
from pytz import UnknownTimeZoneError, timezone

from ..config import get_logger
from ..domain import WeatherData

__all__ = ['AddLocation']
log = get_logger(__name__)


def _warn(message: str):
    messagebox.showwarning(title='Add Location', message=message)


def _error(message: str, error: SystemError):
    log.error(f'{message}\n{error}')
    messagebox.showerror(title='Add Location', message=f'{message} Check the log for more information.')


class AddLocation:
    def __init__(self, parent, location: PyLocation, weather_data: WeatherData):
        self._weather_data = weather_data
        self._is_cancelled = True

        # the editor will update the location with the updated content when it successfully completes
        if not LocationEditor(parent, location, weather_data).is_cancelled():
            try:
                self._weather_data.backend.add_location(location)
                self._is_cancelled = False
            except SystemError as error:
                _error('There was an error adding the location', error)

    def is_cancelled(self) -> bool:
        return self._is_cancelled


class LocationEditor(Dialog):
    def __init__(self, parent, location: PyLocation, weather_data: WeatherData):
        # self._name = None
        # self._name_var = tk.StringVar(parent, value=location.name)

        self._city = None
        self._city_var = tk.StringVar(parent, value=location.city)

        self._state = None
        self._state_var = tk.StringVar(parent, value=location.state)

        self._state_id = None
        self._state_id_var = tk.StringVar(parent, value=location.state_id)

        self._alias = None
        self._alias_var = tk.StringVar(parent, value=location.alias)

        self._latitude = None
        self._latitude_var = tk.StringVar(parent, value=location.latitude)

        self._longitude = None
        self._longitude_var = tk.StringVar(parent, value=location.longitude)

        self._tz = None
        self._tz_var = tk.StringVar(parent, value=location.tz)

        self._location = location
        self._weather_data = weather_data
        self._is_cancelled = True

        super().__init__(parent, title='Add Location')

    def body(self, parent: tk.Frame) -> tk.Entry:
        """Add the dialog fields to the frame supplied by the Dialog."""
        label_options = {"sticky": tk.E, 'padx': (5, 2), 'pady': 5}
        entry_options = {"sticky": tk.W, 'padx': (0, 5), 'pady': 5}

        def mk_entry(row: int, label: str, entry_variable: tk.StringVar, entry_len: int) -> tk.Entry:
            tk.Label(master=parent, text=label).grid(row=row, column=0, **label_options)
            entry = tk.Entry(parent, width=entry_len, textvariable=entry_variable)
            entry.grid(row=row, column=1, **entry_options)
            return entry

        # self._name = mk_entry(0, "Name:", self._name_var, 40)
        self._city = mk_entry(0, "City:", self._city_var, 40)
        self._state = mk_entry(1, "State:", self._state_var, 25)
        self._state_id = mk_entry(2, "State ID:", self._state_id_var, 3)

        def lc_alias(_):
            self._alias_var.set(self._alias_var.get().lower())

        self._alias = mk_entry(3, "Alias:", self._alias_var, 40)
        self._alias.bind("<KeyRelease>", lc_alias)

        self._latitude = mk_entry(4, "Latitude:", self._latitude_var, 20)
        self._longitude = mk_entry(5, "Longitude:", self._longitude_var, 20)
        self._tz = mk_entry(6, "Timezone:", self._tz_var, 20)

        def number_only(action, text):
            if '1' == action:
                for c in text:
                    if not (c.isdigit() or '.' == c or '-' == c or '+' == c):
                        return False
            return True

        number_validator = self.register(number_only)
        self._latitude.configure(validate="key", validatecommand=(number_validator, '%d', '%S'))
        self._longitude.configure(validate="key", validatecommand=(number_validator, '%d', '%S'))

        return self._alias

    def apply(self):
        """Update the location with the contents of the editor."""
        city = self._city_var.get()
        state_id = self._state_id_var.get()
        self._location.city = city
        self._location.state_id = state_id
        self._location.state = self._state_var.get()
        self._location.alias = self._alias_var.get()
        self._location.latitude = self._latitude_var.get()
        self._location.longitude = self._longitude_var.get()
        self._location.tz = self._tz_var.get()
        self._is_cancelled = False

    def validate(self):
        """Called by the Dialog to validate the location contents."""

        if not self._city_var.get():
            _warn('A location city name is required.')
            self.initial_focus = self._city
            return False

        if not self._state_id_var.get():
            _warn('A location state ID is required.')
            self.initial_focus = self._state_id
            return False

        if not self._state_var.get():
            _warn('A location state name is required.')
            self.initial_focus = self._state
            return False

        alias = self._alias_var.get()
        if not alias:
            _warn('An alias name is required.')
            self.initial_focus = self._alias
            return False
        else:
            try:
                if self._weather_data.backend.get_locations(PyLocationFilters([PyLocationFilter(name=alias)])):
                    _warn("The alias is already being used.")
                    return False
            except SystemError as error:
                _error('There was an error validating the location alias.', error)
                return False

        latitude = self._latitude_var.get()
        if not latitude:
            _warn('A latitude is required.')
            self.initial_focus = self._latitude
            return False
        else:
            # make sure the latitude is within bounds
            latitude = float(latitude)
            if latitude < -90.0 or latitude > 90.0:
                _warn('The latitude must be between -90 and 90 degrees.')
                self.initial_focus = self._latitude
                return False

        longitude = self._longitude_var.get()
        if not longitude:
            _warn('A longitude is required.')
            self.initial_focus = self._longitude
            return False
        else:
            # make sure the latitude is within bounds
            longitude = float(longitude)
            if longitude < -180.0 or longitude > 180.0:
                _warn('The must be between -180 and 180 degrees.')
                self.initial_focus = self._longitude
                return False

        tz = self._tz_var.get()
        if not tz:
            _warn('A timezone is required.')
            self.initial_focus = self._tz
            return False
        else:
            try:
                # make sure the timezone matches the pytz zone name
                tzinfo = timezone(tz)
                self._tz_var.set(tzinfo.zone)
            except UnknownTimeZoneError:
                _warn('The timezone does not appear to be valid.')
                self.initial_focus = self._tz
                return False
        return True

    def is_cancelled(self) -> bool:
        return self._is_cancelled
