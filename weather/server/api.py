from datetime import date
from typing import List

from fastapi import APIRouter, FastAPI, HTTPException, Path, Query, Request, Security, status
from fastapi.exception_handlers import http_exception_handler

import weather.domain as wd
from weather import StopWatch
from weather.configuration import get_logger
from .auth import (
    AuthenticationError, DisabledUser, Permissions, SessionExpired, WeatherDataUser,
    auth_router, auth_url, get_current_active_user, get_users
)
from .schemas import DailyHistory, DateRange, HourlyHistory, Location, LocationProperties, User, WeatherHistory

log = get_logger(__name__)

###############################################################################
# Users Api
###############################################################################

_users_router = APIRouter()


@_users_router.get("/",
                   summary="Return information about weather data users.",
                   description="Return information about all weather data users.",
                   response_model=List[User])
async def _get_users(_=Security(get_current_active_user, scopes=[Permissions.ReadUsers])) -> List[User]:
    return [User.from_domain(user) for user in get_users()]


@_users_router.get("/account/",
                   summary="Return information about the current user.",
                   description="Return information about the currently authorized user.",
                   response_model=User)
async def _get_current_user(current_user: WeatherDataUser = Security(get_current_active_user)) -> User:
    return User.from_domain(current_user)


###############################################################################
# Weather Data services
###############################################################################

_weather_data = wd.WeatherData()


async def _get_weather_data(_=Security(get_current_active_user)) -> wd.WeatherData:
    # Depends could be used here as well as Security. The read weather data
    # permission could be specified here but I like the idea of having the
    # permission annotated on the REST endpoints.
    return _weather_data


def initialize_weather_data():
    """
    Weather data currently isn't thread-safe however reading the data will
    pre-load caches. The rest services are currently read-only so seed away.
    """
    log.info("Initialize weather data...")
    stop_watch = StopWatch(label="Complete in")
    _weather_data.preload_data()
    log.info(f'{stop_watch}.')


###############################################################################
# Weather Data Api
###############################################################################

_weather_router = APIRouter()


#
# Locations Api
#


def _get_weather_data_location(name: str, weather_data: wd.WeatherData) -> wd.Location:
    location = weather_data.get_location(name)
    if not location:
        raise HTTPException(status.HTTP_404_NOT_FOUND, f'"{name}" was not found.')
    return location


def _assemble_location(wd_location: wd.Location, properties: bool, histories: bool, weather_data: wd.WeatherData) -> Location:
    dto = Location.from_domain(wd_location)
    if properties:
        dto.properties = LocationProperties.from_domain(weather_data.history_properties(wd_location))
    if histories:
        histories = weather_data.history_date_ranges(wd_location)
        if histories:
            dto.histories = [DateRange.from_domain(dr) for dr in weather_data.history_date_ranges(wd_location)]
    return dto


@_weather_router.get("/",
                     summary="Get summary information about locations.",
                     description="Get a summary of information about weather data locations.",
                     response_model=List[Location],
                     response_model_exclude_unset=True)
async def _get_locations(
        properties: bool = Query(False, description="Optionally include location history statistics."),
        history: bool = Query(False, description="Optionally include a summary of location history."),
        weather_data: wd.WeatherData = Security(_get_weather_data, scopes=[Permissions.ReadWeatherData])
) -> List[Location]:
    return [_assemble_location(location, properties, history, weather_data) for location in weather_data.locations()]


@_weather_router.get("/{name}/",
                     summary="Get summary information about a location.",
                     description="Get a summary of information about a locations weather data.",
                     response_model=Location,
                     response_model_exclude_unset=True)
async def _get_location(
        name: str = Path(..., description="Either the location full name or alias name."),
        properties: bool = Query(False, description="Optionally include location history statistics."),
        history: bool = Query(False, description="Optionally include a summary of location history."),
        weather_data: wd.WeatherData = Security(_get_weather_data, scopes=[Permissions.ReadWeatherData])
) -> Location:
    location = _get_weather_data_location(name, weather_data)
    return _assemble_location(location, properties, history, weather_data)


#
# History Api
#


@_weather_router.get("/{name}/history/",
                     summary="Get complete weather data for a location.",
                     description="Get the dates of available weather data for a location bounded by" +
                                 " a start and end date. The end date defaults to the start date so at most" +
                                 " 1 day of weather history will be returned if the end date is not specified.",
                     response_model=List[date],
                     response_model_exclude_unset=True)
async def _get_location_history_dates(
        name: str = Path(..., description="Location full name or alias."),
        starting: date = Query(None, description='Starting date (YYYY-MM-DD).', alias="from"),
        ending: date = Query(None, description='Ending date (YYYY-MM-DD).', alias="thru"),
        weather_data: wd.WeatherData = Security(_get_weather_data, scopes=[Permissions.ReadWeatherData])
) -> List[date]:
    location = _get_weather_data_location(name, weather_data)
    if not ending:
        ending = starting
    elif ending < starting:
        raise HTTPException(status.HTTP_400_BAD_REQUEST, "From date after thru date.")
    return [hd for hd in weather_data.history_dates(location, starting, ending)]


def _get_weather_history(name: str,
                         starting: date,
                         ending: date,
                         weather_data: wd.WeatherData) -> WeatherHistory:
    location = _get_weather_data_location(name, weather_data)
    if not ending:
        ending = starting
    elif ending < starting:
        raise HTTPException(status.HTTP_400_BAD_REQUEST, "From date after thru date.")
    return WeatherHistory(location=Location.from_domain(location), date_range=DateRange(starting=starting, ending=ending))


@_weather_router.get("/{name}/history/daily/",
                     summary="Get daily weather data.",
                     description="Get daily weather data for a location bounded by a start and end date." +
                                 " The end date defaults to the start date so at most 1 day of weather" +
                                 " history will be returned.",
                     response_model=WeatherHistory,
                     response_model_exclude_unset=True)
async def _get_location_daily_history(
        name: str = Path(..., description="Location full name or alias."),
        starting: date = Query(..., description='History start date (YYYY-MM-DD).', alias="from"),
        ending: date = Query(None, description='History end date (YYYY-MM-DD).', alias="thru"),
        weather_data: wd.WeatherData = Security(_get_weather_data, scopes=[Permissions.ReadWeatherData])
) -> WeatherHistory:
    location = _get_weather_data_location(name, weather_data)
    date_range = wd.DateRange(starting, ending)
    history_dates = weather_data.history_dates(location, date_range.low, date_range.high)
    histories = [DailyHistory.from_domain(daily_history)
                 for daily_history in weather_data.get_history(location, history_dates)]
    return WeatherHistory(location=Location.from_domain(location),
                          date_range=DateRange.from_domain(date_range),
                          daily_histories=histories)


@_weather_router.get("/{name}/history/hourly/",
                     summary="Get hourly weather data.",
                     description="Get hourly weather data for a location bounded by a start and end date." +
                                 " The end date defaults to the start date so at most 1 day of weather" +
                                 " history will be returned.",
                     response_model=WeatherHistory,
                     response_model_exclude_unset=True)
async def _get_location_hourly_history(
        name: str = Path(None, description="Location full name or alias."),
        starting: date = Query(None, description='Starting date (YYYY-MM-DD).', alias="from"),
        ending: date = Query(None, description='Ending date (YYYY-MM-DD).', alias="thru"),
        weather_data: wd.WeatherData = Security(_get_weather_data, scopes=[Permissions.ReadWeatherData])
) -> WeatherHistory:
    location = _get_weather_data_location(name, weather_data)
    date_range = wd.DateRange(starting, ending)
    history_dates = weather_data.history_dates(location, date_range.low, date_range.high)
    histories = [HourlyHistory.from_domain(hourly_history)
                 for hourly_history in weather_data.get_history(location, history_dates, hourly_history=True)]
    return WeatherHistory(location=Location.from_domain(location),
                          date_range=DateRange.from_domain(date_range),
                          hourly_histories=histories)


###############################################################################
# The Weather Data application
###############################################################################


_tags_metadata = [
    {
        "name": "users",
        "description": "Users of the **Weather Data** server."
    },
    {
        "name": "Weather Data",
        "description": "The **Weather Data** REST services."
    }
]

weather_data_app = FastAPI(
    title="Weather Data Services",
    description="A RESTful API providing Weather Data information.",
    openapi_tags=_tags_metadata
)

weather_data_app.include_router(_users_router, prefix="/users", tags=["users"])
weather_data_app.include_router(_weather_router, prefix="/weather", tags=["weather_data"])
weather_data_app.include_router(auth_router, prefix=auth_url)


class _ExceptionHandlers:

    @staticmethod
    async def http_exception_handler(status_code: int, request: Request, error: AuthenticationError):
        return await http_exception_handler(request, HTTPException(
            status_code=status_code,
            detail=error.detail,
            headers={
                "WWW-Authenticate": error.auth_scheme
            }
        ))

    @staticmethod
    @weather_data_app.exception_handler(DisabledUser)
    async def disabled_user_handler(request: Request, error: DisabledUser):
        return await _ExceptionHandlers.http_exception_handler(status.HTTP_403_FORBIDDEN, request, error)

    @staticmethod
    @weather_data_app.exception_handler(SessionExpired)
    async def session_expired_handler(request: Request, error: SessionExpired):
        return await _ExceptionHandlers.http_exception_handler(status.HTTP_403_FORBIDDEN, request, error)

    @staticmethod
    @weather_data_app.exception_handler(AuthenticationError)
    async def authentication_error_handler(request: Request, error: AuthenticationError):
        return await _ExceptionHandlers.http_exception_handler(status.HTTP_401_UNAUTHORIZED, request, error)
