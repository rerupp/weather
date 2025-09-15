# Weather Data GUI
A tkinter based GUI to add and view weather history.

## Introduction
This project provides a cross-platform GUI to display weather history that has
been collected by the Rust weather data project. The GUI is built using
`tkinter`. I'm not a huge fan of the toolkit however it comes with most Python
installations so it does not need to be built for whatever platform you are on.

The `py_weather_lib` project provides the Python bindings to the Rust backend API. I
use PyCharm as the IDE and there is an annoying problem with the editor.
PyCharm does not use the function signature information when generating stubs
files from native extensions. There is an open issue, PY-54189 in YouTrack,
that has been around for 3 years. Definitely not a priority for the PyCharm
team.

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

Create the command using the following command:

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

