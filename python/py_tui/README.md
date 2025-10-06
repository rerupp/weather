# Weather Data TUI
A terminal based UI to view weather history.

## Introduction
This project provides a terminal UI to display weather history that has
been collected by the Rust weather data project. The TUI is built using the
`textual` toolkit.

The API used by Python is within the Rust [weather](../../rust/weather/README.md)
workspace. The [py_lib](../../rust/weather/py_lib/README.md) project contains code that
defines the Python API used to access the Rust [weather_lib](../../rust/weather/lib)
backend. The `py_lib` project has instructions on how to create the `py_weather_lib`
package used by the TUI. The `py_weather_lib` includes Python *type stubs* allowing
an IDE such as PyCharm to understand the `py_weather_lib` API.

### Third Party Packages

There are several 3rd party libraries used by the TUI.

- **`textual[syntax]`** provides the terminal UI framework.
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

Create the `tgui` command from the current directory using the following `pip` command:

```
(venv) c: pip install --editable .
```

#### Dependencies
Here are the primary package dependencies.

| Package    | Version |
|------------|---------|
| textual    | 6.1.0   |
| pytz       | 2025.2  |

