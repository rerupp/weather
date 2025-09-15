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
quickly got out of control and led to the original GUI.

### Current state of the GUI
The current GUI is mostly a port of the original implementation. The biggest difference
is it does not implement the backend data store. Instead it uses the `PyO3` bindings to
call the `Rust` backend which reads and writes weather data history.

It currently lacks functionality to graph weather history for multiple locations. This
will be added in a future release.

### Current state of the TUI
The current TUI can only view weather data history at this point. It is the result of
using the `textual` framework for about a month. I'm pretty pleased with the framework.
It is intuitive, has a rich set of widgets, and has a nice modern look. Startup time 
seems slow to me but once up and running it is fine.

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
