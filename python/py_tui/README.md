# Weather Data TUI
A terminal based UI to view weather history.

## Introduction
This project provides a terminal UI to display weather history that has
been collected by the Rust weather data project. The TUI is built using the
`textual` toolkit.

The `py_weather_lib` project provides the Python bindings to the Rust backend API.
**PyCharm** is the IDE I primarily use to edit code and there is an annoying issue
with the editor. **PyCharm** does not grok function signature information from the
`Pyo3` generated stub files from native extensions. There is an open issue, PY-54189
in YouTrack, that has been around for 3 years. Definitely not a priority for the PyCharm
team. I've avoided using `Python` interface files as suggested by the `Maturin` toolkit
but it is so annoying I'll probably add them in a future release. 

### Third Party Packages

There are several 3rd party libraries used by the GUI.

- **`textual[syntax]`** provides terminal UI framework.
- **`pytz`** provides access to timezone information.

I highly recommend installing `textual-dev` along with the `textual` package. Out of the
box it provides examples and allows capturing `stdout` if problems occur. 

## Installation

Please see the top level [readme](../README.md) for information about how to initially
get started. Before the TUI can be installed the Rust weather library needs
to be built and the `py_weather_lib` package installed into the virtual environment.  

Installation of the TUI is managed by the `setuptools` toolkit. It will install
the TUI package dependencies and creates the console command `tgui` that will
launch the application.

Create the `tgui` command using the following `pip` command:

```
(venv) c: pip install --editable .
```

#### Dependencies
Here are the primary package dependencies.

| Package    | Version |
|------------|---------|
| textual    | 6.1.0   |
| pytz       | 2025.2  |

