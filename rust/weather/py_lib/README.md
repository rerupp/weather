# Python Weather Data Library

A PyO3 based Python library extension for the Rust weather data library.

## Introduction

The project uses PyO3 to create a Python module that interacts
with the native weather data library. The module provides Python objects
that front-end the native weather data library.

Why am I doing this? After the Rust weather data command line tool and
terminal UI was complete I thought it would be fun to build a GUI.
Unfortunately the availability of a Rust crate providing a cross-platform
GUI toolkit is still in the early stages. The original weather data platform
was written in Python and a GUI was built using tkinter.

## Building

The `maturin` tool is used to build the wheel that will be installed into
virtual environments. The following command line will install the wheel into
the current virtual environment.

```
(venv) C: maturin develop
```

Once the command completes there will be a `py_weather_lib` module available
in the virtual environment. You can verify the module is available as
follows.

```
(venv) C: python
>>> import py_weather_lib
>>>
```

## PyO3 Thoughts

Using PyO3 to create the weather data bindings was really straight forward
and easy. Compared to something like a Java JNDI binding PyO3 is trivial.
The way Rust handles memory management gives you confidence there won't be
things like memory leaks.

In Order to get `PyCharm` to understand the bindings, interface files
(`.pyi`) were added to the project. The `Python` interface files are straight
forward to create however it could become a maintenance issue for larger
projects.

Moving data through PyO3 is reasonably quick. Retrieving histories
for 1300 dates in Rust takes about 60ms and through PyO3 about 600ms.
Getting 16 locations takes around 4ms and the location history dates around
14ms.
