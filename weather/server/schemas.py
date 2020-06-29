from datetime import date, datetime
from typing import List, Optional

from pydantic import BaseModel, Field
import pytz

import weather.domain as wd
from .models import WeatherDataUser


class LocationProperties(BaseModel):
    """
    Information about a locations weather data.
    """
    history_count: int = Field(0, description='The count of history data available for the location.')
    raw_history_size: int = Field(0, description='The actual size (in bytes) of history data.')
    compressed_history_size: int = Field(0, description='The size (in bytes) history data actually uses.')
    overall_size: int = Field(0, description='The overall size (in bytes) the location data is using.')

    @staticmethod
    def from_domain(history_properties: wd.WeatherHistoryProperties) -> 'LocationProperties':
        return LocationProperties() if not history_properties else LocationProperties(
            history_count=history_properties.entries,
            raw_history_size=history_properties.entries_size,
            compressed_history_size=history_properties.compressed_size,
            overall_size=history_properties.size
        )


class DateRange(BaseModel):
    """
    A range of dates associate with weather data.
    """
    starting: date = Field(..., title='Starting date', description="The starting date.")
    ending: date = Field(None, title='Ending date', description="The ending date (inclusive).")

    @staticmethod
    def from_domain(date_range: wd.DateRange) -> 'DateRange':
        return DateRange(starting=date_range.low, ending=date_range.high)


class Location(BaseModel):
    """
    Information about a location whose weather data can be queried.
    """
    full_name: str = Field(..., description='The full location name ("{name}, {state}").')
    name: str = Field(..., description='The location name.')
    state: str = Field(..., description='The location state (OR, AZ, etc.)')
    alias: str = Field(..., description='An alias or nickname associated with the location.')
    longitude: str = Field(..., description='The location longitude.')
    latitude: str = Field(..., description='The location latitude.')
    tz: str = Field(..., description='The location timezone.')
    properties: LocationProperties = Field(None,
                                           title="location properties",
                                           description='Optional information about resources being consumed' +
                                                       ' by history data.')
    histories: List[DateRange] = Field(None,
                                       title='history date ranges',
                                       description='Optional history dates available for the location.')

    @staticmethod
    def from_domain(location: wd.Location) -> 'Location':
        name, state = location.name.split(",")
        return Location(
            name=name.strip(),
            state=state.strip(),
            full_name=location.name,
            alias=location.alias,
            longitude=location.longitude,
            latitude=location.latitude,
            tz=location.tz
        )


class DailyHistory(BaseModel):
    """
    Historical weather data containing day-by-day conditions for a location.
    """
    time: date = Field(..., description='The calendar date associated with the weather data.')
    high_temp: float = Field(None, description='The daytime high temperature.')
    high_ts: datetime = Field(None, description='The time of day when the high temperature occurred.')
    low_temp: float = Field(None, description='The overnight low temperature.')
    low_ts: datetime = Field(None, description='The time of day when the low temperature occurred.')
    max_temp: float = Field(None, description='The maximum temperature for the given date.')
    max_ts: datetime = Field(None, description='The time of day when the maximum temperature occurred.')
    min_temp: float = Field(None, description='The minimum temperature for the given date.')
    min_ts: datetime = Field(None, description='The time of day when the minimum temperature occurred.')
    wind_speed: float = Field(None, description='The wind speed in miles per hour.')
    wind_gust: float = Field(None, description='The wind gust speed in miles per hour.')
    wind_gust_ts: datetime = Field(None, description='The time of day when the wind gust occurred.')
    wind_bearing: int = Field(None, description='The direction wind is coming from.')
    cloud_cover: float = Field(None, description='The percentage of sky occluded by clouds (between 0 and 1 inclusive).')
    humidity: float = Field(None, description='The relative humidity (between 0 and 1 inclusive).')
    dew_point: float = Field(None, description='The dew point in degrees.')
    sunrise_ts: datetime = Field(None, description='The time of day when the sun will rise.')
    sunset_ts: datetime = Field(None, description='The time of day when the sun will set.')
    uv_index: int = Field(None, description='The maximum UV index.')
    uv_index_ts: datetime = Field(None, description='The time of day when the maximum UV index occurred.')
    summary: str = Field(None, description='A description of the weather data for the date.')

    @staticmethod
    def from_domain(history: dict) -> 'DailyHistory':
        def get(key: wd.DailyWeatherContent):
            return history.get(key.value)

        return DailyHistory(
            time=wd.DataConverter.to_binary_date(get(wd.DailyWeatherContent.TIME), pytz.utc),
            high_temp=get(wd.DailyWeatherContent.TEMPERATURE_HIGH),
            high_ts=wd.DataConverter.to_binary_datetime(get(wd.DailyWeatherContent.TEMPERATURE_HIGH_TIME), pytz.utc),
            low_temp=get(wd.DailyWeatherContent.TEMPERATURE_LOW),
            low_ts=wd.DataConverter.to_binary_datetime(get(wd.DailyWeatherContent.TEMPERATURE_LOW_TIME), pytz.utc),
            max_temp=get(wd.DailyWeatherContent.TEMPERATURE_MAX),
            max_ts=wd.DataConverter.to_binary_datetime(get(wd.DailyWeatherContent.TEMPERATURE_MAX_TIME), pytz.utc),
            min_temp=get(wd.DailyWeatherContent.TEMPERATURE_MIN),
            min_ts=wd.DataConverter.to_binary_datetime(get(wd.DailyWeatherContent.TEMPERATURE_MIN_TIME), pytz.utc),
            wind_speed=get(wd.DailyWeatherContent.WIND_SPEED),
            wind_gust=get(wd.DailyWeatherContent.WIND_GUST),
            wind_gust_ts=wd.DataConverter.to_binary_datetime(get(wd.DailyWeatherContent.WIND_GUST_TIME), pytz.utc),
            wind_bearing=get(wd.DailyWeatherContent.WIND_BEARING),
            cloud_cover=get(wd.DailyWeatherContent.CLOUD_COVER),
            humidity=get(wd.DailyWeatherContent.HUMIDITY),
            dew_point=get(wd.DailyWeatherContent.DEW_POINT),
            sunrise_ts=wd.DataConverter.to_binary_datetime(get(wd.DailyWeatherContent.SUNRISE_TIME), pytz.utc),
            sunset_ts=wd.DataConverter.to_binary_datetime(get(wd.DailyWeatherContent.SUNSET_TIME), pytz.utc),
            uv_index=get(wd.DailyWeatherContent.UV_INDEX),
            uv_index_ts=wd.DataConverter.to_binary_datetime(get(wd.DailyWeatherContent.UV_INDEX_TIME), pytz.utc),
            summary=get(wd.DailyWeatherContent.SUMMARY)
        )


class HourlyHistory(BaseModel):
    """
    Historical weather data containing hour-by-hour conditions for a location.
    """
    time: datetime = Field(..., description='The timestamp associated with the weather data.')
    temperature: float = Field(None, description='The air temperature in degrees.')
    apparent_temperature: float = Field(None, description='The "feels like" temperature in degrees.')
    wind_speed: float = Field(None, description='The wind speed in miles per hour.')
    wind_gust: float = Field(None, description='The wind gust speed in miles per hour.')
    wind_bearing: int = Field(None, description='The direction wind is coming from.')
    cloud_cover: float = Field(None, description='The percentage of sky occluded by clouds (between 0 and 1 inclusive).')
    humidity: float = Field(None, description='The relative humidity (between 0 and 1 inclusive).')
    dew_point: float = Field(None, description='The dew point in degrees.')
    uv_index: int = Field(None, description='The UV index.')
    summary: str = Field(None, description='A description of the weather data.')

    @staticmethod
    def from_domain(history: dict) -> 'HourlyHistory':
        def get(key: wd.HourlyWeatherContent):
            return history.get(key.value)

        return HourlyHistory(
            time=wd.DataConverter.to_binary_datetime(get(wd.HourlyWeatherContent.TIME), pytz.utc),
            temperature=get(wd.HourlyWeatherContent.TEMPERATURE),
            apparent_temperature=get(wd.HourlyWeatherContent.APPARENT_TEMPERATURE),
            wind_speed=get(wd.HourlyWeatherContent.WIND_SPEED),
            wind_gust=get(wd.HourlyWeatherContent.WIND_GUST),
            wind_bearing=get(wd.HourlyWeatherContent.WIND_BEARING),
            cloud_cover=get(wd.HourlyWeatherContent.CLOUD_COVER),
            humidity=get(wd.HourlyWeatherContent.HUMIDITY),
            dew_point=get(wd.HourlyWeatherContent.DEW_POINT),
            uv_index=get(wd.HourlyWeatherContent.UV_INDEX),
            summary=get(wd.HourlyWeatherContent.SUMMARY)
        )


class WeatherHistory(BaseModel):
    """
    The server response for API calls requesting access to a location's historical weather data.
    """
    location: Location = Field(..., description="The location associated with the history data.")
    date_range: DateRange = Field(..., description="The starting and ending history dates.")
    daily_histories: List[DailyHistory] = Field(None, description="The optional daily weather histories")
    hourly_histories: List[HourlyHistory] = Field(None, description="The optional hourly weather histories")


class User(BaseModel):
    """
    Information about the users of weather data.
    """
    username: str = Field(..., description='The user name to user when authorizing access to weather data.')
    email: Optional[str] = Field(None, description='The email address associated with the user.')
    full_name: Optional[str] = Field(None, descrption='The full name of the user.')
    disabled: Optional[bool] = Field(False, description='Marks the user as unable to access weather data.')
    permissions: Optional[List[str]] = Field(None, description="The permissions available to the user.")

    @staticmethod
    def from_domain(user: WeatherDataUser) -> 'User':
        if user:
            return User(
                username=user.username,
                email=user.email,
                full_name=user.full_name,
                disabled=user.disabled,
                permissions=user.permissions
            )


class Token(BaseModel):
    access_token: bytes
    token_type: str
