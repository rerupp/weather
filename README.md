# Historical Weather Data

A collection of applications to gather and display historical weather data.

## Background

The original weather data project was written entirely in Python. It started out while
spending the winter in AZ. During several happy hours, long time visitors (snow birds),
kept mentioning it was the coldest winter they could remember. Some of them had
been coming to AZ for upwards to 20 years.

Not that I didn't believe what they were saying, but I began to wonder if there was some online
service that would provide historical weather data. At the time I came across a site, *Dark
Sky*, that had a `REST` API providing daily weather history. Originally the weather data
was a collection of scripts with Excel but it quickly became a mess to get and show data. This
is when the original Python implementation began.

In 2020 *Dark Sky* was purchased by Apple and the API went away. That mostly ended the Python
project. Other services at that time were either ones you had to pay for or the free services
just didn't provide the same amount of information.

Back in May 2022 I came across an article that discussed the Linux communities decision to allow
Rust to be used within the kernel. I had read several articles on the language and was somewhat
surprised due to how (relatively) new the language was.

I spent some time going through *The Rust Programming Language* and came away wanting to explore
it in more detail. If I could come up with some type of project I could take her out for a spin
so to speak.

I wondered if there were any new weather data services available online and came across the *Visual
Crossing* site. Similar to *Dark Sky*, access to the site was free (depending on usage) and
it provided data similar to what had already collected with the Python programs. Thats when
the Rust version of the Python implementation began.

## The <code>weather</code> Project

The Python and Rust projects were previously maintained in separate repositories on `GitHub`.
The original projects have been retired now that they are combined into this project.

The internal [rust](./rust/README.md) project contains the implementation of weather data. It 
provides a CLI that can initialize, add, and report weather history. It is the primary interface
to weather data right now.

The internal [python](./python) project implements several frontend interfaces. One is a TUI
application built using `textual`. The other project is a GUI build using the `Tk` library
included with the `python` distribution. The project uses `PyO3` bindings to access the `rust`
library implementing weather data. Information on how to install the bindings can be
found in the [PyO3](./rust/py_weather_lib) project.

## Getting Started

The scripts and examples will reflect command line development on Windows 11. I've spot checked
building on WSL2 but currently the majority of my development time is spent under Windows.

Once the following steps are complete review the *Getting Started* sections in the
[python](python/README.md) and [rust](rust/README.md) folders.

### Python Virtual Environment

Create a Python virtual environment in the repository root directory. The `Python` code base 
requires at least version 3.10. I have been running exclusively on version 3.13 and will move to 
version 3.14 at some point in the future. 

> Technically the `rust` source code does not require Python however editors such as IntelliJ
> *RustRover* can use it when editing the `py_weather_lib` type file.

From the root of the repository run the `initialize.cmd` script. The following listing shows
the steps taken to create a virtual environment.

```
C:>initialize.cmd

C:>py -3.13-64 -m venv .venv
...

C:>call .venv\Scripts\activate.bat
...

(.venv) C:>python.exe -m pip install --upgrade pip
...

(.venv) C:>pip install maturin
...

(.venv) C:>pip install setuptools
...

(.venv) C:>pip install setuptools-scm
...

(.venv) C:>pip install importlib-resources
...
```

### Visual Crossing License (Optional)

If you intend to try and capture historical weather data you will need to
visit [Visual Crossing](https://www.visualcrossing.com/) and get a license. There is a "sign up
for free" link that will let you retrieve up to 1000 days of historical data
per calendar day.
