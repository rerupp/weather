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
runner, gui runner and REST services runner. It works well and support both
invocation from the command line and PyCharm.

Once the repository has been cloned issue the following commands depending
on the OS you are running:

##### Linux
```shell script
python -m venv venv
source venv/bin/activate
pip install --editable .
```
Once the commands successfully complete the `venv` directory will
contain a `cli`, `gui` and `server` script that will run the CLI, GUI
and REST services respectively. 
##### Windows 10.
```shell script
python -m venv venv
venv\Scripts\activate
pip install --editable .
```
Once the command successfully complete the 'venv' Scripts directory will
contain a `cli.exe` and `gui.exe` the CLI and GUI respectively.
     
#### Dependencies
Here are the package dependencies installed via `setup.py` and the versions
being used.

|Package|Version|
|---|---|
|pytz|2019.3 (Windows) 2020.1 (Linux)|
|PyYAML|5.3.1|
|requests|2.23.0|
|tkcalendar|1.6.1|
|tksheet|4.6.1 (Windows) 4.8.2 (Linux)|
|Click|7.1.2|
|fastapi|0.58.0|
|pydantic|1.5.1|
|PyJWT|1.7.1|
|uvicorn|0.11.5|
|python-nultipart|0.0.5|
|passlib|1.7.2|
|orjson|3.1.1|

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
* `weather.server` contains the REST api and server.

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

### The `server` Package
While I was playing with the `Raspberry Pi` there was this idea in the back of
my mind about building out a greenhouse controller and being able to monitor
and control it from my phone, tablet, or notebook. The controller would be
connected to my home network and I would have a REST API to access it. What
a pleasant surprise to come across `FastAPI`.

Going through the `FastAPI` tutorial it seemed pretty straight forward to add a
read-only REST frontend to the weather data domain. I also wanted to give the
`OAuth2` integration a try. I've created several implementations of security
with `Spring` and I was curious to see how they compared.

I was pleasantly surprised by the `FastAPI` framework. While not as rich as
`Spring` it certainly is _much_ easier to use. The integration with `pydantic`
and `OpenApi` (formerly Swagger) made it really easy to bring up the frontend
and test without building out a client. The framework support for `OAuth2` made
it really easy (and fast development wise) to implement authentication and
permissions. *Soooo* much easier than `Spring Security`. 

Here are the service highlights:

* There are two (2) categories of services.
    * User services. All users can access information about their own account
     however the other services are gated by a _user:read_ permission.
    * Weather data services are gated by a _weather_data:read_ permission.
* Authentication is managed by the server. There are 3 users defined out of the
box and the password is the same as the username.
    * The **admin** user is allowed _user:read_ and _weather_data:read_
     permission.
    * The **user** user is allowed _weather_data:read_ permission.
    * The **guest** user is allowed _weather_data:read_ permissions however the
    user is disabled and will not be able to authorize.
* Permissions are granted to a user using `OAuth2` scopes, sent in as part
of the authentication process. The authentication token returned by the services
will be valid for 5 minutes.
* The `OpenApi` integration is on by default and can be accessed by going to
the `/docs` endpoint once the server is running.
* The `server` script starts the REST services and has command line options to
select the hostname and port to use. A side note on the script implementation.
I came across `click`, decided to give it a try, and came away liking the
framework. The declarative approach is a nice alternative compared to
`argparse`.

I originally was thinking about using `SQLAlchemy` combined with `SQLite` but...
I'm not really a big fan of ORM (Object Relational Mapper) tools especially
having fought `Hibernate` over the years. I much prefer document based data
stores and `SQLite` has less than spectacular write speeds. I have the code
stashed away and will probably include it in another push but right now
preloading the 10k+ histories takes around 1.5 seconds so really why care...
 
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
