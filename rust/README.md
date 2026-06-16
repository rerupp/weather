# Rust Weather

The rust weather project collects and displays historical weather data for some location
based on its latitude and longitude. Currently US and Canada city databases are supported
but any location in the world can be set up and used.

## Why I Am Doing All Of This

This all began as a way to familiarize myself with Rust. I was looking for some project I could
dive into to explore the language and decided porting the Python weather project I built several
years ago would be a good choice.

After the console based Rust frontends were running and the backend became stable I began to
consider building a GUI. Rust does have cross-platform GUI toolkits however the choices in Python  
are much better. This resulted in the PyO3 [py_weather_lib](py_weather_lib/README.md) library that
exposes [WeatherData](weather_lib/src/weather_data.rs) to Python.

### Impressions

As expected the Rust only applications are very performant. The database is currently
implemented on top of Sqlite3 and performance for this single user use case is great.
The backend filesystem storage performance, using ZIP archives, is acceptable although
a threaded archive reader was required to make it that way. Scanning the contents of
25 location archives that contain approximately 250k days of weather history is down
in the 1 to 2 second range,

### Background

The original project started in the `rust_playground` repository. When `Python` was introduced
as a weather data font-end, having the `Python` code exist in the `rust_playground` seemed odd
so the `weather` repository was created and the `rust` code was copied over into it.

### Release Notes

The current release notes can be viewed [here](RELEASE_NOTES.md).

## Project Structure

The weather project is a Rust cargo based workspace consisting of the CLI mainline and supporting
libraries. It has a dependency on the `toolslib` crate.

### [cli](cli/README.md) Directory

This directory contains the <code>weather</code> CLI program.

### [weather_lib](weather_lib/README.md) Directory

This directory is the **weather_lib** package and contains the primary API and backend
implementations of historical weather data.

### [py_weather_lib](py_weather_lib/README.md) Directory

This directory is the **py_weather_lib** package and contains `PyO3` bindings used by the `Python`
front-ends.

### [termui_lib](termui_lib/README.md) Directory

This directory is the **termui_lib** package and contains low level components used to build
the old <code>weather</code> terminal UI interface.

***The project is no longer part of the workspace. I've left it in the repository because it
has code samples that are still useful.***

### [tui_lib](termui_lib/README.md) Directory

This directory is the **tui_lib** package and contains the low level components used to build the
viewport based terminal UI interface.

## Getting Started

There really isn't much to do in order to get things going. Follow the Rust install
directions and everything else is straight forward.

Here are the steps to bootstrap the weather CLI and weather history data backend (Windoz version).

```
$ cargo build
...
    Finished `dev` profile [unoptimized + debuginfo] ...
$ set PATH=%PATH%;%CD%\target\debug
$ weather -h
The weather data command line.

Usage: weather [OPTIONS] <COMMAND>
...
$ mkdir weather_data
$ weather admin init 
$ weather admin uscities --init --load=resources\uscities.csv
```

Use the following command to create the `weather.toml` configuration file in the current
directory. Edit the file and change the Visual Crossing `api-key` from UNAVAILABLE to the one
you were assigned when registering for the timeline API.

```
$ weather admin config --init
$ type weather.toml
[weather-data]
directory = "weather_data"
fs-only = false
max-workers = 32

[visual-crossing]
endpoint = "https://weather.visualcrossing.com/VisualCrossingWebServices/rest/services/timeline"
api-key = "UNAVAILABLE"

[us-cities]
filename = "uscities.csv"
```

A US city can be found and added using the following commands.

```
$ weather lc yuma
City, Region      Latitude/Longitude        Timezone
------------ --------------------------- ---------------
Yuma, AZ           32.5995/-114.5491     America/Phoenix
Yuma, CO           40.1235/-102.7161     America/Denver
Yuma, TN           35.8438/-88.3372      America/Chicago
$ weather al --city=Yuma --cn="United States" --cc=US --rn=Arizona --rc=AZ --lat=32.5995 --lng=-114.5491 --tz=utc yuma
$ weather ll
Alias City, Region      Latitude/Longitude     Timezone
----- ------------ --------------------------- --------
yuma  Yuma, AZ           32.5995/-114.5491     UTC
$ weather ml --tz=america/phoenix yuma
The following updates were made:
  tz='America/Phoenix'
$ weather ll
Alias City, Region      Latitude/Longitude        Timezone
----- ------------ --------------------------- ---------------
yuma  Yuma, AZ           32.5995/-114.5491     America/Phoenix
```

History can be added and viewed using the following commands.

```
$ weather ah yuma dec-2025 dec-15-2025
...
15 histories added

$ weather ah yuma dec-2025
.....
31 histories added
$ weather lh
Alias Location        History Dates
----- -------- ----------------------------
yuma  Yuma, AZ Dec-01-2025 thru Dec-15-2025
$ weather ls
Alias Location Days Store Size DB Size Overall Size
----- -------- ---- ---------- ------- ------------
yuma  Yuma, AZ   15      5 KiB   5 KiB       10 KiB
===== ======== ==== ========== ======= ============
Total            15      5 KiB   5 KiB       10 KiB
$ weather rh yuma dec-2025
           ----- Temperature -----  Dew
   Date    High      Low     Mean  Point
---------- ----- ----------- ----- -----
2025-12-01  71.1     56.0     62.5  36.0
2025-12-02  71.1     48.8     59.0  28.6
2025-12-03  70.2     45.0     58.1  32.4
2025-12-04  66.8     47.9     57.6  29.8
...
```

## Build Environment

I haven't built on WSL2 for a while but here's information about the toolchain on Windoz.

```
$ rustup show
Default host: x86_64-pc-windows-msvc
rustup home:  ...

installed toolchains
--------------------
stable-x86_64-pc-windows-msvc (active, default)

active toolchain
----------------
name: stable-x86_64-pc-windows-msvc
active because: it's the default toolchain
installed targets:
  x86_64-pc-windows-msvc

$ rustup --version
rustup 1.29.0 (28d1352db 2026-03-05)
info: This is the version for the rustup toolchain manager, not the rustc compiler.
info: the currently active `rustc` version is `rustc 1.96.0 (ac68faa20 2026-05-25)`
```

### *Documentation*

If you're going to build documentation I would suggest using the following `cargo` command.

```
cargo doc --workspace --no-deps --document-private-items
```

## Dependencies

Here is a list of the external crate workspace dependencies.

| Crate      | Version |      Features      |
|:-----------|:--------|:------------------:|
| chrono     | 0.4     |       serde        |
| chrono_tz  | 0.10    |       serde        |
| crossterm  | 0.29    |                    |
| log        | 0.4     |                    |
| rusqlite   | 0.40    |  bundled, chrono   |
| serde      | 1       |       derive       | 
| serde_json | 1       |   preserve_order   |
| ratatui    | 0.30    | all-widgets, serde |
| toml       | 1.0     |   preserve_order   |
