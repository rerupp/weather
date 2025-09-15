# Weather Data

The weather data project collects and displays historical weather data for some location
based on its latitude and longitude. Primarily it is for cities in the US but any location
in the world can be set up and used.

## Why Am I Doing This???

This project started as a way to familiarize myself with Rust. I was looking for some project to
build and decided emulating the `Python` weather project I built several years ago would be
a good start.

The performance of the `Python` apps were reasonable however the `Rust` implementation, as one
would expect, is much quicker. Even though the `Rust` GUI front-end is based on `Python`
and `Tk`, performance is many times better than the `Python` only version.

### Background

The original project started in the `rust_playground` repository. When `Python` was introduced 
as a weather data font-end, having the `Python` code exist in the `rust_playground` seemed odd 
so the `weather` repository was created and the `rust` code was copied over into it.

### Release History

#### Current Release (Sep-2025)
This is a copy of the latest version in the `rust_playground`. The `rust_playground` will 
continue to live on but weather history code will no longer be changed there.

## Project Structure

The weather project is a `cargo` based workspace consisting of the CLI mainline and supporting
libraries. It has a dependency on the `toolslib` crate.

### `cli` Directory

This directory contains the source code for the CLI mainline.

### `lib` Directory

This directory contains the backend implementation of the weather domain.

### `py_lib` Directory

This directory contains the `PyO3` bindings used by the `Python` front-ends.

### `termui` Directory

This directory contains the low level components used to build the CLI UI interface.

## Getting Started

There really isn't much to do in order to get things going. Follow the Rust install
directions and everything else is straight forward.

Here are the steps to get started (from the `rust` directory).

```
$ cargo build
$ mkdir weather_data
$ target\debug\weather admin uscities --load=resources\uscities.csv
$ target\debug\weather tui 
```

From the main window press `ALT-N`, followed by `ALT-S`, followed by `ALT-U`. This will bring
up the US Cities search dialog allowing you to add a location. Once you have a location added
you can press `ENTER` while on the location to bring up a menu that will allow you to add
or report weather history.

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
rustup 1.28.2 (e4f3ad6f8 2025-04-28)
info: This is the version for the rustup toolchain manager, not the rustc compiler.
info: The currently active `rustc` version is `rustc 1.88.0 (6b00bc388 2025-06-23)`
```

### *Documentation*

If you're going to build documentation I would suggest using the following `cargo` command.

```
cargo doc --workspace --no-deps --document-private-items
```

## Dependencies

Here are a list of workspace dependencies.

| Crate             | Version |       Features        |
|:------------------|:--------|:---------------------:|
| chrono            | 0.4     |         serde         |
| chrono_tz         | 0.10    |         serde         |
| clap              | 4.5     |        derive         |
| crossterm         | 0.28.1  |                       |
| log               | 0.4     |                       |
| rusqlite          | 0.32    | blob, bundled, chrono |
| serde             | 1       |        derive         | 
| serde_json        | 1       |    preserve_order     |
| ratatui           | 0.28    |  all-widgets, serde   |
| reqwest           | 0.11    |       blocking        |
| toml              | 0.8     |    preserve_order     |
| sql_query_builder | 2.4     |        sqlite         |
| strum             | 0.26    |        derive         |