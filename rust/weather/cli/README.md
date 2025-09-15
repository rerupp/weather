# Weather Data CLI

This module contains the weather data command line interface. The command line entry point in
the `main` program. It is responsible to parsing the command line, bootstrapping the
`WeatherData` API, and executing commands.

Errors that happen are captured by `main` and written to `stderr`. It uses `ExitStatus` to
return the success or failure of the commands.

## The `cli` Module

The `cli` module contains the definition of the commands. It is built on top of the `clap` crate
using their program API. Each command exposes a `NAME` attribute, a command `get()` function that 
returns the command parser, and an `execute()` function that runs the command.

The bootstrapping of the CLI include initialization a logger. The verbosity of the logger  
(`INFO`, `DEBUG`, or `TRACE`) can be changed via a command line argument. The default level is  
`WARN`.

Most of the commands support producing report output in the form of plain text, `JSON`, or `CSV`
formats. Command line arguments allow the reports to be saved to a file instead of being output  
to `stdout`.

### Modules with `cli`

#### The `admin` module

The `admin` module contains the administration CLI commands.

#### The `reports` module

The `reports` module contains the various reports available to the CLI commands. The `reports` 
module is shared between the CLI and terminal UI.

#### The `tui` module

The `tui` module contains the terminal based UI application. It relies on the `termui` library 
to provide components such as dialogs and widgets.

The TUI main window consists of a menu bar with tabbed windows showing the locations, summary, or
history reports. Only textual report output is available as of now. Weather data location can be
added along with historical weather data.

#### The `user` module

The `user` module contains the non-administration CLI commands.

## The `weather` command line.

The weather executable consists of various subcommands. If a subcommand is not entered, a help
overview is provided.

```
$ weather
The weather data command line.

Usage: weather [OPTIONS] <COMMAND>

Commands:
  ll     List the known weather data history locations_win.
  lh     List the dates of weather history available by location.
  ls     List a summary of weather data available by location.
  rh     Generate a weather history report for a location.
  ah     Add weather history to a location.
  qc     Search cities for location information.
  tui    A Terminal based weather data UI.
  admin  The weather data administration tool.
  help   Print this message or the help of the given subcommand(s)

Options:
  -c, --config <FILE>    The configuration file pathname (DEFAULT weather.toml).
  -d, --directory <DIR>  The weather data directory pathname.
      --fs               Do not use a weather history DB if one is available.
  -l, --logfile <FILE>   The log filename (DEFAULT stdout).
  -a, --append           Append to the logfile, otherwise overwrite.
  -v, --verbose...       Logging verbosity (once=INFO, twice=DEBUG, +twice=TRACE)
  -h, --help             Print help
  -V, --version          Print version
```

Help for subcommands are also available.

```
$ weather rh
Generate a weather history report for a location.

Usage: weather rh [OPTIONS] <LOCATION> <FROM> [THRU]

Arguments:
  <LOCATION>  The location to use for the weather history.
  <FROM>      The weather history starting date.
  [THRU]      The weather history ending date.

Options:
  -t, --temp           Include temperature information in the report (default).
  -p, --precip         Include percipitation information in the report.
  -c, --cnd            Include weather conditions in the report.
  -s, --sum            Include summary information in the report.
  -a, --all            Include all weather information in the report.
      --text           The report will be plain Text (default)
      --csv            The report will be in CSV format.
      --json           The report will be in JSON format.
  -P, --pretty         For JSON reports output will be pretty printed.
  -r, --report <FILE>  The report filename (default stdout).
  -A, --append         Append to the report file, otherwise overwrite.
  -h, --help           Print help
```

#### `admin` commands.

The available administrative commands can be listed as shown below..

```
$ weather admin
The weather data administration tool.

Usage: weather admin <COMMAND>

Commands:
  init      Initialize the weather data database.
  drop      Delete the existing database schema.
  reload    Reload database weather history for locations.
  show      Show information about the weather data backend components.
  uscities  Administer the US Cities database.
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

Help for subcommands are also available.

```
$ weather admin init -h
Initialize the weather data database.

Usage: weather admin init [OPTIONS]

Options:
      --threads <THREADS>  The number of threads to use [default: 8]
      --drop               Drops the database before initializing.
      --load               Load the database after initializing.
  -h, --help               Print help
  ```
