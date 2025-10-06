# Weather Data GUI
A tkinter based GUI to add and view weather history.

## Introduction
This project provides a cross-platform GUI to display weather history that has
been collected by the Rust weather data project. The GUI is built using
`tkinter`. I'm not a huge fan of the toolkit however it comes with most Python
installations so it does not need to be built for whatever platform you are on.


The API used by Python is within the Rust [weather](../../rust/weather/README.md)
workspace. The [py_lib](../../rust/weather/py_lib/README.md) project contains code that
defines the Python API used to access the Rust [weather_lib](../../rust/weather/lib)
backend. The `py_lib` project has instructions on how to create the `py_weather_lib`
package used by the TUI. The `py_weather_lib` includes Python *type stubs* allowing
an IDE such as PyCharm to understand the `py_weather_lib` API.

### Third Party Widgets

There are several 3rd party libraries used by the GUI.

- **`tkcalendar`** provides a Calendar widget for date entry.
- **`matplotlib`** provides the graph widget used for data display.
- **`pytz`** provides access to timezone information.
- **`tzdata`** provides the timezone information database.
- **`PyYAML`** provides support for reading and writing YAML files.

## Installation

Please see the top level [readme](../README.md) for information about how to initially
get started. Before the GUI can be installed the Rust weather library needs
to be built and the `py_weather_lib` package installed into the virtual environment.  

Installation of the GUI is managed by the `setuptools` toolkit. It will install
the GUI package dependencies and creates the console command `wgui` that will
launch the application.

Create the command from the current directory using the following command:

```
(venv) c: pip install --editable .
```

#### Dependencies
Here are the primary package dependencies.

| Package    | Version |
|------------|---------|
| matplotlib | 3.10    |
| numpy      | 2.2     |
| pytz       | 2024.2  |
| PyYAML     | 6.0.2   |
| tkcalendar | 1.6.1   |
| tzdata     | 2024.2  |

