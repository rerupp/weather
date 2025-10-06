# Weather Data
A python based terminal UI and graphical UI to view and update weather history.

## Background
The original `Python` project started out many years ago as a way to re-familiarize
myself with Python 3 in hopes of doing something useful with a Raspberry PI.
While wintering down in AZ, after several happy hour conversations, long time snow
birds kept saying it was the coldest winter they could remember.

This led me to see if there was a way to look at weather history for locations and
graph temperature trends across the years. I started out using simple scripts to call
the rest services, store data, and create the files that were imported into Excel. It
quickly got out of control and led to the first generation of weather data. The
implementation was a full stack application that including both frontend and backend.

Several years later Rust caught my attention. In order to explore Rust I decided  
to create a port the Python backend and CLI into Rust. Performance of the Rust backend
was many times faster than the Python version which led me to explore how
the Python frontend ecosystem could be used with the Rust backend. Thats when I found
`PyO3` which has led to the current Python frontends.

## Overview

The Python frontends do not directly interact with the historical weather data storage.
Instead the frontends use a `PyO3` based API to access and update the historical
weather data. The [py_lib](../rust/weather/py_lib/README.md) project within the Rust
`weather` workspace defines the API. Using `maturin` a `py_weather_lib` package
can be created that contains the Python weather data API.


### Current state of the GUI
The current GUI is mostly a port of the original implementation. The Python code
that was used to access weather data storage has been replaced `py_weather_lib`.

Historical weather data can be added, reports generated, and graphs created. Currently
the graphing implementation lacks the ability to graph weather history for multiple
locations. 

### Current state of the TUI
The TUI is built using the `textual` framework. It has functionality to add and
report historical weather data. The plan is to expand the TUI to help with bootstrapping
the weather data storage and add graphing functionality.

## Installation
I have mostly been developing with Windows 11 so installation will be based on that
platform. The `Python` code depends on the version to be at least 3.10. Both TUI and
GUI rely on `setuptools` to create the respective executables.

Run the following commands from the current directory.

### Windows 11
```shell script
py -3.13-64 -m venv .vevn
.venv\Scripts\activate.bat
(.venv): pip install maturin
(.venv): pip install setuptools
(.venv): pip install setuptools-scm
(.venv): pip install importlib-resources
```

### Create the `PyO3` Bindings
From the `python` directory run the following commands.

```shell script
(.venv): cd ..\rust\weather\py_lib
(.venv): maturin develop
```

## Project Structure
The TUI and GUI are split into separate folder structures.

### `py_gui`
This folder contains the GUI implementation. See the [readme](./py_gui/README.md) file for
instructions on how to install the executable.

### `py_tui`
This folder contains the TUI implementation. See the [readme](./py_tui/README.md) file for
instructions on how to install the executable.
