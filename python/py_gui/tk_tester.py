import logging
import tkinter as tk
import tkinter.ttk as ttk

from weather_data import DataCriteria, WeatherConfig, create as create_rust_bindings

from py_gui.config import initialize as config_initialize
from py_gui.domain import WeatherData
from py_gui.tk_weather.history_graph import HistoryGraph
from py_gui.tk_weather.infrastructure import WeatherView


def run():
    root = tk.Tk()
    root.title("Report View Tester")
    root.configure(bg='light grey')
    root.geometry("640x480")

    root.columnconfigure(0, weight=1)
    root.rowconfigure(0, weight=1)

    # set up the notebook
    notebook = ttk.Notebook(root)
    notebook.columnconfigure(0, weight=1)
    notebook.rowconfigure(0, weight=1)
    notebook.grid(sticky=tk.NSEW)

    ttk.Sizegrip(root).grid(row=1, sticky=tk.SE)

    config_initialize(logfile=None, log_level=logging.DEBUG)
    # weather_config = WeatherConfig(dirname='../../weather/weather_data')
    weather_config = WeatherConfig(dirname='../../../weather_data')
    weather_data = WeatherData(create_rust_bindings(weather_config))
    # location = weather_data.get_locations(DataCriteria(filters=['sherwood']))[0]
    # locations = weather_data.get_locations(DataCriteria(filters=['tigard']))[0]
    locations = weather_data.get_locations(DataCriteria(filters=['tigard', 'medford', 'roseburg', 'kfalls']))

    # make sure the existing graph is replaced
    def add_tab(name: str, weather_view: WeatherView):
        if notebook.index('end'):
            notebook.forget(0)
        notebook.add(weather_view.view(), text=name)

    HistoryGraph(notebook, locations, weather_data, add_tab)
    root.mainloop()


if __name__ == '__main__':
    run()
