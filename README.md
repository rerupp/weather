# Weather Data
A python CLI and GUI tool for tracking weather history.

## Background
This project started out as a way to re-familiarize myself with Python 3 in
hopes of doing something useful with a Raspberry PI. While down in AZ last
winter, in several happy hours, I kept hearing from long time snow birds it
was the coldest winter they could remember.

I heard that so much I decided to see if there was a way to look at weather
history for someplace and graph temperature trends across the years.
I did some research and found Dark Sky has a rest API I could call that
provides historical weather data and it was for free depending on my usage
(_see note 1_). It seemed like Python would be a good fit.

I started out using simple scripts to call the rest services, store data,
and create the files that I imported into Excel. It quickly became a problem
when someone asked "Well what about here or how do temperatures compare between
here and there". Hence a couple of hours a day for seven months and here is
the result.

## Installation

The `setuptools` package was a pleasant surprise. I have not tried to
bundle a package but I do use it to get dependencies, create the cli
runner, and gui runner. It works well and support both invocation from the
command line and PyCharm.

Once the repository has been cloned issue the following commands depending
on the OS you are running:

##### Linux
```shell script
python -m venv venv
source venv/bin/activate
pip install --editable .
```
Once the commands successfully complete the `venv/bin` directory will
contain a `cli` and `gui` script that run the CLI and GUI respectively 
##### Windows 10.
```shell script
python -m venv venv
venv\Scripts\activate
pip install --editable .
```
Once the command successfully complete the `venv\Scripts` directory will
contain a `cli.exe` and `gui.exe` the CLI and GUI respectively.
     
### Dependencies
Here are the package dependencies and the version being used.

|Package|Version|
|---|---|
|pytz|2019.3 (Windows) 2020.1 (Linux)|
|PyYAML|5.3.1|
|requests|2.23.0|
|tkcalendar|1.6.1|
|tksheet|4.6.1 (Windows) 4.8.2 (Linux)|

### Running/Debugging from PyCharm
Once I moved to `setuptools` and removed the previous runners I had to
change the run configuration in PyCharm to execute the scripts in the `venv`
directory. On Windows 10 you will need to use the `cli-script.py` or
`gui-script.py` files and not the `*.exe` files. Make sure to set the 
working directory to the repository root before applying the change.

## Python 3
I've run the CLI and GUI on the following versions of Python:

* 3.8.0 (Windows 10)
* 3.8.2 (Fedora 32 and Windows 10)
* 3.8.3 (Fedora 32)

Before diving into the packages and their description I want to put in a plug
for Python 3. I've used Python for many, many years to support scripting
utilities on Un*x platforms. This is my first deep dive into Python 3 and
I 've come away with a better appreciation of the language, where it is in the
3.8 release and where it's going.

For me, concepts in the language that most comes to mind are.

* Array and dictionary comprehensions. This is so cool and the amount of
 boiler plate code crap it eliminates makes me smile.
* Named tuples and dataclasses. Having a read-only object makes it really
easy to pass data around without worry about modifications or copying.
* The support for Generics and abstract classes. While I didn't make great
use of either I was happy to see the language providing the support. Having
a collection (no pun intended) of classes that help extend containers such
as arrays and dictionaries was sorely needed. 
* The typing system. Before I hear the groans go out, I really learned to
appreciate the feature and use it a lot. It doesn't prohibit the ability to
still use duck typing but when teamed up with intelligent IDE's such as
PyCharm it really makes development nice.
* Lastly performance. It is addicting to make a change, run or debug,
without going through a build step. Some of the file parsing done to bring up
the application or a dialog is almost transparent to the launch time. I have
weather data for several dozen cites and locations with up to 10 years history
and the GUI application memory runtime footprint is around 80MiB. That's when
caching around 75% of the historical data. Impressive for an interpretive
language.

## Package Structure

The `weather` directory contains all of the weather data packages.

* `weather` contains utilities that really didn't belong to one package.
* `weather.cli` contains the CLI client. 
* `weather.configuration` contains settings and configuration data.
* `weather.domain` contains the weather data domain model and objects.
* `weather.gui` contains the GUI client.

### The `cli` Package
The CLI client is built on top of Python `argparse`. Similar to tools like
`svn` it contains sub-commands that run various weather data commands. It was
the first client I built and works well but you need something like Excel
to graph the data.  

### The `configuration` Package
This package contains configuration and settings options for weather data. It
is used internally by the other weather data packages. It has an internal
`data` package that contains the supported colors and default settings. The
text data is read via the `importlib` library available in Python 3.

### The `domain` Package
This package contains the domain and objects used to store weather data.
The size of weather history for a day is pretty small considering. I didn't
want to use a database so I decided a directory and archive data store would
be fine for the usage and amount of data being collected.

The `domain` package contains an internal `data` package that holds the known
cities data. Similar to the configuration package, data is read via the
`importlib` library.

By default all weather data is stored in the `weather_data` directory in the
current working directory. The following table describes files within the
folder.

|File|Description|
|---|---|
|`locations.json`|The cities (or locations) that can use the Dark Sky API|
|`city/location.zip`|Weather data for an entry in the `locations.json` file|

The city/location archive contains weather data files by date. The name of
the archive entry reflects both the city/location and the data.

### The `gui` Package
The GUI came about because it became a pita to run the CLI to create a
csv file that needed to be imported into Excel to create a temperature graph.
It is implemented using `tkinter` so there are no dependencies on other
libraries such a `gtk` or `qt`.

This was the first time I really took a deep dive into `tkinter`. I'm a
backend enterprise services architect but I've written UI and components with
X11, Java, node.js, C#, and win32 in previous lives. I came away with an
appreciation for `tkinter` and what can be done with Python and the library.
Yes its a dated UI and lacks widgets that would really be helpful but it works
and is fast. I thought about other graphical tool kits but I did most of the
coding on my wife's laptop over this past winter and did not want to install
a collection of other software libraries. I felt bad enough putting
Python 3 and PyCharm on her laptop.

### Where are the unit tests?
There aren't any actually. I started out with building them out but the
architecture was moving so quickly it became a pita to maintain them. There
is an `examples.py` file that has a history testing functionality but that's
all. Honestly there are people I worked with that will probably go 'What???'.
Regardless I do want to explore unit testing in Python, it looks interesting.
The real problem is I'm pretty jazzed right now about `.net core` on Linux,
WSL 2 on Windows 10, and doing some `C#` development. 

---

_Note 1_: Unfortunately Dark Sky was purchased by Apple and the API will no
longer be available at the end of 2020. I'll decide if the tool is useful
enough for me to use another weather data provider. It should be fairly straight
forward moving to another API if they provide similar information (yeah,
that's the architect in me speaking...).
