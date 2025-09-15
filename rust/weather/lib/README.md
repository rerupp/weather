# `weather_lib` Library

The `weather_lib` library implements the historical weather data API and administration
of the data storage.

## Overview

The `WeatherData` structure contains the public facing API to access historical weather data. The 
`Backend` is used by the `WeatherData` API to read and write historical weather data. There are
currently two (2) implementations of the `Backend`. One is file system based using ZIP archives 
and the other is a database implementation built using `SQlite3`.

## Module Overview

### The `admin` module.

This module contains the administrative API `WeatherAdmin` and the entities specific 
to weather data administration.

### The `backend` Module

The `backend` module defines the `Backend` trait. Within the module are the filesystem and 
database implementations. 

Regardless of the implementation historical weather data is always stored into `Zip`
archives. This allows data to be easily backed up and reloaded as changes are made to the data 
model.

#### The `backend::filesys` module.

This module contains support for the files used in weather data. It implements `Zip` file
archive reading and writing along with the weather locations `JSON` document. It also contains
operating system independent implementations for weather data directories and files.

#### The `backend::db` module.

This module contains support for the database implementation of weather data history. It
also uses the `filesys` module to update the weather history archives and locations document
as changes are made.

### The `entities` module.

This module contains all the structures used to implement weather data commands.

### The `history_client` module.

This module contains the `HistoryClient` used to get historical weather data. The client
is built on top of the `reqwest` crate and contains the *Visual Crossing* `Rest` client 
implementation.

### The `weather_data` module.

This module contains the `WeatherData` API.
