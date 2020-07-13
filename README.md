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
##### Windows 10.
```shell script
python -m venv venv
venv\Scripts\activate
pip install --editable .
```
Once the commands have completed the following weather data utilities will be
available:
- `wcli`: a command line utility to add and view weather data.
- `wgui`: a GUI based on tkinter allowing weather data to be added and viewed.
- `wserver`: a web server providing read-only REST services.
- `dbload`: a command line utility that will load weather data into a database
(sqlite is currently supported).
- `dbcli`: a command line utility allowing weather data to be read from a
database.
     
#### Dependencies
Here are the package dependencies installed via `setup.py` and the versions
being used.

|Package|Version|
|---|---|
|Click|7.1.2|
|fastapi|0.58.0|
|orjson|3.1.1|
|passlib|1.7.2|
|pydantic|1.5.1|
|PyJWT|1.7.1|
|pytz|2019.3 (Windows) 2020.1 (Linux)|
|PyYAML|5.3.1|
|python-multipart|0.0.5|
|requests|2.23.0|
|SQLAlchemy|1.3.17
|tkcalendar|1.6.1|
|tksheet|4.6.1 (Windows) 4.8.2 (Linux)|
|uvicorn|0.11.5|

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
* `weather.db` contains a simple database implementation built using
`sqlalchemy`.

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

By default all weather data is stored in the current working directory
under a folder named `weather_data`. The following table describes files
within the folder.

|File|Description|
|---|---|
|`locations.json`|The cities (or locations) that can use the Dark Sky API|
|`city/{location-alias}.zip`|The weather data storage for a location.|

Given the following weather data locations as displayed by `wcli`.
```shell script
$ wcli ll
------Location------- ----Alias----- ---Longitude/Latitude---- -----Timezone------
Boise, ID             boise_id          -116.2312/43.6007      America/Boise
Carson City, NV       carson_city_nv    -119.7474/39.1511      America/Los_Angeles
Fortuna Foothills, AZ foothills      -114.4118901/32.6578355   America/Phoenix
Indio, CA             indio          -116.2188054/33.7192808   America/Los_Angeles
Klamath Falls, OR     kfalls            -121.7754/42.2191      America/Los_Angeles
Lake Havasu City, AZ  havasu         -114.3224495/34.4838502   America/Phoenix
Lake Oswego, OR       lake_oswego_or    -122.7003/45.4129      America/Los_Angeles
Las Cruces, NM        las_cruces_nm     -106.7893/32.3265      America/Denver
Las Vegas, NV         vegas          -115.1485163/36.1672559   America/Los_Angeles
Medford, OR           medford           -122.8537/42.3372      America/Los_Angeles
Mesa, AZ              mesa           -111.8314773/33.4151117   America/Phoenix
Roseburg, OR          roseburg          -123.3518/43.2232      America/Los_Angeles
Seattle, WA           seattle        -122.3300624/47.6038321   America/Los_Angeles
St. George, UT        stgeorge       -113.5841313/37.104153    America/Denver
Tigard, OR            tigard            -122.7845/45.4237      America/Los_Angeles
Tucson, AZ            tucson         -110.9748477/32.2228765   America/Phoenix
```
The `weather_data` folder would have the following content.
```shell script
]$ ls -go weather_data
total 25320
-rw-r--r--. 1 2035697 Jul 10 08:54 boise_id.zip
-rw-rw-r--. 1 2093135 Jul 10 08:30 carson_city_nv.zip
-rw-rw-r--. 1 2044268 Jun  1 12:39 foothills.zip
-rw-rw-r--. 1 1977714 Jun 28 12:34 havasu.zip
-rw-rw-r--. 1 2034949 Jun 28 12:32 indio.zip
-rw-rw-r--. 1 2029631 Jul  8 09:25 kfalls.zip
-rw-rw-r--. 1   56062 Jun  1 12:39 lake_oswego_or.zip
-rw-rw-r--. 1 1592860 Jun  1 12:39 las_cruces_nm.zip
-rw-rw-r--. 1    2693 Jul  4 09:05 locations.json
-rw-rw-r--. 1 2004853 Jul  8 09:20 medford.zip
-rw-rw-r--. 1 2038098 Jun  1 12:39 mesa.zip
-rw-rw-r--. 1 1980186 Jul  7 08:58 roseburg.zip
-rw-rw-r--. 1  145678 Jun  1 12:39 seattle.zip
-rw-rw-r--. 1 1680247 Jul 10 13:28 stgeorge.zip
-rw-rw-r--. 1 2065225 Jul  7 08:56 tigard.zip
-rw-rw-r--. 1 2035792 Jun  1 12:39 tucson.zip
-rw-rw-r--. 1   53839 Jun  1 12:39 vegas.zip
```
Details about the weather data storage can be viewed using the following `wcli`
command.
```shell script
$ wcli ls
      Location        Overall Size History Count Raw History Size Compressed Size
--------------------- ------------ ------------- ---------------- ---------------
Boise, ID                1,988 kiB         1,284       11,543 kiB       1,815 kiB
Carson City, NV          2,044 kiB         1,284       11,640 kiB       1,841 kiB
Fortuna Foothills, AZ    1,996 kiB         1,274       13,839 kiB       1,820 kiB
Indio, CA                1,987 kiB         1,274       13,945 kiB       1,830 kiB
Klamath Falls, OR        1,982 kiB         1,284       11,557 kiB       1,819 kiB
Lake Havasu City, AZ     1,931 kiB         1,274       13,923 kiB       1,770 kiB
Lake Oswego, OR             55 kiB            31          308 kiB          50 kiB
Las Cruces, NM           1,556 kiB         1,061        9,204 kiB       1,392 kiB
Las Vegas, NV               53 kiB            31          304 kiB          49 kiB
Medford, OR              1,958 kiB         1,284       11,398 kiB       1,790 kiB
Mesa, AZ                 1,990 kiB         1,274       13,975 kiB       1,839 kiB
Roseburg, OR             1,934 kiB         1,284       11,445 kiB       1,761 kiB
Seattle, WA                142 kiB            92          850 kiB         130 kiB
St. George, UT           1,641 kiB         1,061       11,403 kiB       1,498 kiB
Tigard, OR               2,017 kiB         1,284       11,870 kiB       1,854 kiB
Tucson, AZ               1,988 kiB         1,274       14,004 kiB       1,826 kiB
===================== ============ ============= ================ ===============
Totals                  25,262 kiB        16,350      161,208 kiB      23,082 kiB
```

### The `gui` Package
The GUI came about because it became a pita to run the CLI to create a
csv file that needed to be imported into Excel to create a temperature graph.
It is implemented using `tkinter` so there are no dependencies on other
libraries such a `gtk` or `qt`.

This was the first time I really took a deep dive into `tkinter`. I'm a
backend enterprise services architect however I have written UI and
components using X11, Java, node.js, C#, and WIN32 in previous lives.
I came away with an appreciation for `tkinter` and what can be done with Python
using the library. Yes its a dated UI and lacks widgets that would really be
helpful but it is cross platform (ok, well kinda) and reasonably fast. I
thought about other graphical tool kits but I did most of the
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

#### REST services
I was pleasantly surprised by the `FastAPI` framework. While not as rich as
`Spring` it certainly is _much_ easier to use. The integration with `pydantic`
and `OpenApi` (formerly Swagger) made it really easy to bring up the frontend
and test without building out a client.
 
All service users must be authenticated. The framework support for `OAuth2` made
it really easy (and fast development wise) to implement authentication and
permissions. *Soooo* much easier than `Spring Security`. 

##### User services
The user services provide read-only access to information about users of the
system. It also has a hook to redirect requests to the login endpoint if
a request does not include a `OAuth2` security header. A session will last
5 minutes before needing to refresh.

There are two (2) permissions or `OAuth2` scopes gating access to services.
* A _user:read_ permission allows listing all users of the server. All users
can see their account information without permission.
* A _weather_data:read_ permission is required to access the weather data
services.

The following table provides information about the service users. A users
password is the same as the username.

|username|Permissions|Disabled|
|---|---|---|
|admin|_user:read_ _weather_data:read_||
|user|_weather_data:read_||
|guest|_weather_data:read_|Yes|

##### Weather data services    
The weather data services provide read-only access to locations and weather
data history. The services for getting daily or hourly data should support
paging but don't. Considering the use case, requesting 6 months of hourly
data, the payload is reasonably small for my home network.

##### The web server and `OpenApi` support
The `wserver` script starts the REST services and has command line options to
select the _hostname_ and _port_ to use. A side note on the script
implementation. I came across `click`, decided to give it a try, and came away
liking the framework. The declarative approach is a nice alternative compared
to `argparse`.

The `OpenApi` integration is on by default and can be accessed by going to
the `/docs` endpoint. 

### The `db` package
When I went through the `fastAPI` tutorial I came across `sqlalchemy`. I'm
not a huge fan of ORM frameworks having battled `Hibernate` in several 
past lifetimes, however I thought it would be fun to try `sqlalchemy` out.
Creation of the data models were straight forward and seemed to be less of a
headache than `Hibernate` building from scratch.

#### The database schemas
Ok, I'm not a dba and I don't profess to be one. What I wanted to do was load
some data that wasn't just a couple of tables holding weather data. With that
in mind I decided to create data models that would include both the REST
server users and the weather data history.

##### User data schema
The user data model consist of three tables:
* The `users` table has user information.
* The `permissions` table permission information.
* The `user_permissions` table links users to permissions.

##### Weather data schema
The weather data model consists of four tables:
* The `locations` table has information about a location.
* The `daily` table contains normalized daily history for a location. A
foreign key ties daily history to the associated location.
* The `hourly` table contains normalized hourly history for a location. A
foreign key ties hourly history to the associated location.
* The `history` table contains both daily and hourly history for a
location. The daily and hourly history are stored in separate columns as
JSON blobs.
 
The use case for weather data is find daily or hourly history for a location
for a given date range. There really isn't a driving need to have normalized
data at this point because the history date is the filter not content of
the history data.

##### Normalize data or not
I decided to keep the normalize `daily` and `hourly` tables so I could measure
difference in data size and time to load. As expected storing data as JSON
blobs in the `history` table took about 3 times as much space as normalized
tables. Data load times for the `history` table however is about 6 times faster
(10 seconds versus ~1 minute). Time to access data is faster with the
normalized tables however loading 6 months of history and measuring a time of
40 ms versus 60 ms is not a concern at this point. Being able to load data in
~10 seconds versus ~1 minute? To me that is much more significant.

The `dbcli` utility has a command that will show approximate table usage in
the `sqlite` database. Here's a sample of output from the command.
```shell script
$ dbcli ls
      Location        Histories Daily Table Hourly Table History Table Tables Combined
--------------------- --------- ----------- ------------ ------------- ---------------
Boise, ID                 1,284     216 kiB    3,201 kiB    10,291 kiB      13,709 kiB
Carson City, NV           1,284     216 kiB    3,201 kiB    10,291 kiB      13,709 kiB
Fortuna Foothills, AZ     1,274     215 kiB    3,177 kiB    10,211 kiB      13,602 kiB
Indio, CA                 1,274     215 kiB    3,177 kiB    10,211 kiB      13,602 kiB
Klamath Falls, OR         1,284     216 kiB    3,201 kiB    10,291 kiB      13,709 kiB
Lake Havasu City, AZ      1,274     215 kiB    3,177 kiB    10,211 kiB      13,602 kiB
Lake Oswego, OR              31       5 kiB       77 kiB       248 kiB         331 kiB
Las Cruces, NM            1,061     179 kiB    2,645 kiB     8,504 kiB      11,328 kiB
Las Vegas, NV                31       5 kiB       77 kiB       248 kiB         331 kiB
Medford, OR               1,284     216 kiB    3,201 kiB    10,291 kiB      13,709 kiB
Mesa, AZ                  1,274     215 kiB    3,177 kiB    10,211 kiB      13,602 kiB
Roseburg, OR              1,284     216 kiB    3,201 kiB    10,291 kiB      13,709 kiB
Seattle, WA                  92      15 kiB      229 kiB       737 kiB         982 kiB
St. George, UT            1,061     179 kiB    2,645 kiB     8,504 kiB      11,328 kiB
Tigard, OR                1,284     216 kiB    3,201 kiB    10,291 kiB      13,709 kiB
Tucson, AZ                1,274     215 kiB    3,177 kiB    10,211 kiB      13,602 kiB
===================== ========= =========== ============ ============= ===============
Totals                   16,350   2,754 kiB   40,767 kiB   131,048 kiB     174,569 kiB
```

#### A note about sqlite
The biggest issue with the database is write locking. Originally I had the
`dbload` utility threaded and immediately hit *database locked* errors. I
considered creating a queue with a worker thread that would write data however
this would not be issues if I was using a backend such as `Postgres` or
`Oracle` so why do it here. Database load time is around 60 seconds when
including the normalized tables which is fine. 

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
