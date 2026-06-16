# weather_lib Library

This library contains the backend implementation for historical weather data.

## Overview

There are two entry points into the library.
* [WeatherAdmin](src/admin.rs) is the API used to manage historical weather data.
* [WeatherData](src/weather_data.rs) is the API used to view and update historical weather data.

## Module Overview

### The [admin](src/admin.rs) module.

This module implements the [WeatherAdmin](src/admin.rs) API. It also contains the entities 
specific to weather data administration.

### The [backend](src/backend.rs) Module

Internally [WeatherData](src/weather_data.rs) uses a [Backend](src/backend.rs) trait to access
weather data. This module defines that trait and exposes the API used to create implementations.

#### The [filesys](src/backend/filesys.rs) module.

This module contains a file based implementation of the [Backend](src/backend.rs) trait.
*JSON* document files are used to store properties and metadata such as location information
and configuration data. *Zip* archives are used to store a locations weather history.

#### The [sqlite](src/backend/db/sqlite.rs) module.

This module contains a database implementation of the [Backend](src/backend.rs) trait. It uses
the [filesys](src/backend/filesys.rs) implementation as a backing store to allow data to be
easily backed up and the database rebuilt as required.

### The [configuration](src/configuration.rs) module.

This module provides the API to access and update weather data configuration properties.

### The [entities](src/entities.rs) module.

This module contains all the datat structures used by the weather data API.

### The [histories_future](src/histories_future.rs) module.

This module contains the API that manages collecting historical weather data. It is a thread 
based `future` that can be queried for completion and data. Access to the Visual 
Crossing timeline service and conversion of the response is handled by the background 
thread.

### The [weather_data](src/weather_data.rs) module.

This module implements the [WeatherData](src/weather_data.rs) API. It also exposes the API used 
to create instances of the API.
