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

## Getting Started
As with the other project I have been primarily developing on Windows 11 so installation 
will be based on that platform. Installation of the TUI and GUI rely on `setuptools` to create 
the respective executables.

The `initialize.cmd` script will assemble and install the GUI and TUI into the virtual 
environment. The following listing shows the steps taken to install them.

```
(.venv) C:>initialize.cmd

(.venv) C:>call PyO3.cmd
...

(.venv) C:>pip install --editable py_gui
...
Successfully installed py_gui-1.0.0

(.venv) C:>pip install textual
...

(.venv) C:>pip install textual-dev
...

(.venv) C:>pip install --editable py_tui
...
Successfully installed py_tui-1.0.0
```

### Create the `PyO3` Bindings

The [PyO3.cmd](./PyO3.cmd) script will create the [py_weather_lib](../rust/py_weather_lib/README.md) 
bindings Python clients call to access the native Rust implementation. The following listings 
shows the steps taken to create the bindings.

```
(.venv) C:>PyO3.cmd

(.venv) C:>pushd C:..\rust\py_weather_lib

(.venv) C:\Users\rncru\dev\weather\rust\py_weather_lib>maturin develop
...

(.venv) C:>popd
```

## Project Structure
The TUI and GUI are split into separate folder structures.

### `py_gui`
This folder contains the GUI implementation. See the [readme](./py_gui/README.md) file for
instructions on how to install the executable.

### `py_tui`
This folder contains the TUI implementation. See the [readme](./py_tui/README.md) file for
instructions on how to install the executable.
