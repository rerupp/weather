"""
There really should be no reason to expose the table model to callers.
If you do need the actual model id then add a specific CRUD operation
so the use case can be tracked.
"""
from datetime import date, datetime
from enum import Enum
from typing import Any, Dict, Generator, Iterator, List, Optional, TypeVar

import orjson
import pytz
from sqlalchemy.orm import Session

import weather.domain as wd
import weather.server as ws
from weather.configuration import get_logger
from .models import DailyHistory, History, HourlyHistory, Location, Permission, User

_log = get_logger(__name__)
T = TypeVar('T')


def db_refresh(session: Session, db_data: List[T]) -> List[T]:
    for data in db_data:
        session.refresh(data)
    return db_data


###############################################################################
# permissions
###############################################################################


class PermissionCRUD:

    @staticmethod
    def get_permissions(session: Session) -> List[ws.Permissions]:
        return [PermissionMAP.to_domain(p) for p in Permission.get_all(session)]

    @staticmethod
    def add_permissions(session: Session, permissions: List[ws.Permissions], refresh=True) -> List[ws.Permissions]:
        permissions_to_add: List[Permission] = []
        for permission in permissions:
            if Permission.get(session, permission.value):
                _log.debug('Permission "%s" already exists...', permission.name)
            else:
                permissions_to_add.append(PermissionMAP.from_domain(permission))
        if permissions_to_add:
            Permission.add(session, permissions_to_add)
        if refresh:
            db_refresh(session, permissions_to_add)
        return [PermissionMAP.to_domain(p) for p in permissions_to_add]


get_permissions = PermissionCRUD.get_permissions
add_permissions = PermissionCRUD.add_permissions


class PermissionMAP:

    @staticmethod
    def from_domain(permission: ws.Permissions) -> Permission:
        return Permission(name=permission.value)

    @staticmethod
    def to_domain(permission: Permission) -> ws.Permissions:
        # PyCharm is confused because it groks the permission attributes as column not string
        # noinspection PyArgumentList
        return ws.Permissions(permission.name)


###############################################################################
# users
###############################################################################


class UserCRUD:

    @staticmethod
    def add(session: Session, users: Iterator[ws.WeatherDataUser], refresh=True) -> List[ws.WeatherDataUser]:
        permissions = [p for p in Permission.get_all(session)]
        users_to_add: List[User] = []
        for user in users:
            if User.exists(session, user.username):
                _log.warning(f'"{user}" already exists.')
            else:
                users_to_add.append(UserMap.from_domain(user, permissions))
        if users_to_add:
            User.add(session, users_to_add)
        if refresh:
            for user in users_to_add:
                session.refresh(user)
        return [UserMap.to_domain(u) for u in users_to_add]

    @staticmethod
    def get(session: Session, username: str) -> ws.WeatherDataUser:
        user = User.get(session, username)
        if user:
            return UserMap.to_domain(user)

    @staticmethod
    def get_all(session: Session) -> List[ws.WeatherDataUser]:
        return [UserMap.to_domain(u) for u in User.get_all(session)]


add_users = UserCRUD.add
get_user = UserCRUD.get
get_users = UserCRUD.get_all


class UserMap:

    @staticmethod
    def from_domain(user: ws.WeatherDataUser, permissions: List[Permission]) -> User:
        def to_permission(name: str) -> Permission:
            for _permission in permissions:
                if _permission.name == name:
                    return _permission
            raise ValueError(f'"{name} is not in the list of permissions.')

        return User(
            username=user.username,
            hashed_password=user.hashed_password,
            email=user.email,
            full_name=user.full_name,
            disabled=user.disabled,
            permissions=[to_permission(p.value) for p in user.permissions]
        )

    @staticmethod
    def to_domain(user: User) -> ws.WeatherDataUser:
        return ws.WeatherDataUser(
            username=user.username,
            hashed_password=user.hashed_password,
            email=user.email,
            full_name=user.full_name,
            disabled=user.disabled,
            permissions=[
                PermissionMAP.to_domain(p) for p in user.permissions
            ]
        )


###############################################################################
# locations
###############################################################################


class LocationCRUD:

    @staticmethod
    def add(session: Session, locations: Iterator[wd.Location], refresh=True) -> List[wd.Location]:
        locations_to_add: List[Location] = []
        for location in locations:
            if Location.exists(session, location.name) or Location.exists(session, location.alias):
                _log.warning(f'"{location.name}"/{location.alias} already exists.')
            else:
                locations_to_add.append(LocationMAP.from_domain(location))
        if locations_to_add:
            Location.add(session, locations_to_add)
        if refresh:
            db_refresh(session, locations_to_add)
        return [LocationMAP.to_domain(location) for location in locations_to_add]

    @staticmethod
    def get(session: Session, name: str) -> wd.Location:
        location = Location.get(session, name)
        if location:
            return LocationMAP.to_domain(location)

    @staticmethod
    def get_all(session: Session, order_by_name=True) -> List[wd.Location]:
        return [LocationMAP.to_domain(location) for location in Location.get_all(session, order_by_name)]


add_locations = LocationCRUD.add
get_location = LocationCRUD.get
get_locations = LocationCRUD.get_all


class LocationMAP:

    @staticmethod
    def from_domain(location: wd.Location) -> Location:
        full_name = location.name
        name, state = full_name.split(",")
        return Location(
            full_name=full_name,
            name=name.strip(),
            state=state.strip(),
            alias=location.alias,
            longitude=location.longitude,
            latitude=location.latitude,
            tz=location.tz
        )

    @staticmethod
    def to_domain(location: Location) -> wd.Location:
        return wd.Location(
            name=location.full_name,
            alias=location.alias,
            longitude=location.longitude,
            latitude=location.latitude,
            tz=location.tz
        )


###############################################################################
# History
###############################################################################


class HistoryCRUD:

    @staticmethod
    def add_daily_histories(session: Session,
                            location: wd.Location,
                            histories: Iterator[dict],
                            refresh=True) -> List[dict]:
        pass

    @staticmethod
    def add_hourly_histories(session: Session,
                             location: wd.Location,
                             histories: Iterator[dict],
                             refresh=True) -> List[dict]:
        pass

    # noinspection DuplicatedCode
    @staticmethod
    def add(session: Session, location: wd.Location, histories: Iterator[wd.FullHistory], refresh=True):
        db_location = Location.get(session, location.name)
        if not db_location:
            _log.warning(f'"{location.name}" was not found.')
        else:
            db_histories: List[History] = [HistoryMAP.from_domain(db_location, history) for history in histories]
            History.add(session, db_histories)
            if refresh:
                db_refresh(session, db_histories)

    # noinspection DuplicatedCode
    @staticmethod
    def add_daily(session: Session, location: wd.Location, histories: Iterator[wd.FullHistory], refresh=True):
        db_location = Location.get(session, location.name)
        if not db_location:
            _log.warning(f'"{location.name}" was not found.')
        else:
            db_histories: List[DailyHistory] = [DailyHistoryMAP.from_domain(db_location, history) for history in histories]
            DailyHistory.add(session, db_histories)
            if refresh:
                db_refresh(session, db_histories)

    @staticmethod
    def add_hourly(session: Session, location: wd.Location, histories: Iterator[wd.FullHistory], refresh=True):
        db_location = Location.get(session, location.name)
        if not db_location:
            _log.warning(f'"{location.name}" was not found.')
        else:
            # this comprehension flattens the list of hourly history for each history date
            db_histories: List[HourlyHistory] = [
                hourly_history for history in histories
                for hourly_history in HourlyHistoryMAP.from_domain(db_location, history)
            ]
            HourlyHistory.add(session, db_histories)
            if refresh:
                db_refresh(session, db_histories)

    @staticmethod
    def get_history_dates(session: Session, location: wd.Location) -> List[date]:
        db_location = Location.get(session, location.name)
        if not db_location:
            _log.warning(f'"{location.name}" was not found.')
        else:
            return History.get_dates(session, db_location)

    @staticmethod
    def get_daily_history(session: Session,
                          location: Location,
                          date_range: Optional[wd.DateRange] = None) -> Generator[wd.FullHistory, None, None]:
        db_location = Location.get(session, location.name)
        if not db_location:
            _log.warning(f'"{location.name}" was not found')
        else:
            for history in History.get_daily(session, db_location, date_range):
                yield HistoryMAP.to_domain(history)


add_daily_histories = HistoryCRUD.add_daily
add_histories = HistoryCRUD.add
add_hourly_histories = HistoryCRUD.add_hourly
get_daily_history = HistoryCRUD.get_daily_history
get_history_dates = HistoryCRUD.get_history_dates


class HistoryMAP:

    @staticmethod
    def from_domain(location: Location, history: wd.FullHistory) -> History:
        return History(
            location_id=location.id,
            date=wd.DataConverter.to_binary_date(history.date, pytz.utc),
            daily=orjson.dumps(history.daily),
            hourly=orjson.dumps(history.hourly)
        )

    @staticmethod
    def to_domain(history: History) -> wd.FullHistory:
        return wd.FullHistory(
            date=history.date,
            daily=orjson.loads(history.daily) if history.daily else None,
            hourly=orjson.loads(history.hourly) if history.hourly else None
        )


E = TypeVar('E', str, Enum)


class HistoryConverter:
    """Yes, I know this can be managed by just using enum for the argument..."""

    def __init__(self, history: Dict[str, Any]):
        self.history = history
        self.as_str = lambda k: k.value if isinstance(k, Enum) else k

    def to_date(self, key: E) -> date:
        return wd.DataConverter.to_binary_date(self.history.get(self.as_str(key)), pytz.utc)

    def to_datetime(self, key: E) -> datetime:
        return wd.DataConverter.to_binary_datetime(self.history.get(self.as_str(key)), pytz.utc)

    def to_float(self, key: E) -> float:
        return wd.DataConverter.to_binary_float(self.history.get(self.as_str(key)))

    def to_int(self, key: E) -> int:
        return wd.DataConverter.to_binary_int(self.history.get(self.as_str(key)))

    def to_str(self, key: E) -> str:
        value = self.history.get(self.as_str(key))
        return value if isinstance(value, str) else str(value) if value else ""


class DailyHistoryMAP:

    @staticmethod
    def from_domain(location: Location, history: wd.FullHistory) -> DailyHistory:
        converter = HistoryConverter(history.daily)
        return DailyHistory(
            location_id=location.id,
            time=converter.to_date(wd.DailyWeatherContent.TIME),
            high_temp=converter.to_float(wd.DailyWeatherContent.TEMPERATURE_HIGH),
            high_ts=converter.to_int(wd.DailyWeatherContent.TEMPERATURE_HIGH_TIME),
            low_temp=converter.to_float(wd.DailyWeatherContent.TEMPERATURE_LOW),
            low_ts=converter.to_int(wd.DailyWeatherContent.TEMPERATURE_LOW_TIME),
            max_temp=converter.to_float(wd.DailyWeatherContent.TEMPERATURE_MAX),
            max_ts=converter.to_int(wd.DailyWeatherContent.TEMPERATURE_MAX_TIME),
            min_temp=converter.to_float(wd.DailyWeatherContent.TEMPERATURE_MIN),
            min_ts=converter.to_int(wd.DailyWeatherContent.TEMPERATURE_MIN_TIME),
            wind_speed=converter.to_float(wd.DailyWeatherContent.WIND_SPEED),
            wind_gust=converter.to_float(wd.DailyWeatherContent.WIND_GUST),
            wind_gust_ts=converter.to_int(wd.DailyWeatherContent.WIND_GUST_TIME),
            wind_bearing=converter.to_int(wd.DailyWeatherContent.WIND_BEARING),
            cloud_cover=converter.to_float(wd.DailyWeatherContent.CLOUD_COVER),
            humidity=converter.to_float(wd.DailyWeatherContent.HUMIDITY),
            dew_point=converter.to_float(wd.DailyWeatherContent.DEW_POINT),
            sunrise_ts=converter.to_int(wd.DailyWeatherContent.SUNRISE_TIME),
            sunset_ts=converter.to_int(wd.DailyWeatherContent.SUNSET_TIME),
            uv_index=converter.to_int(wd.DailyWeatherContent.UV_INDEX),
            uv_index_ts=converter.to_int(wd.DailyWeatherContent.UV_INDEX_TIME),
            summary=converter.to_str(wd.DailyWeatherContent.SUMMARY)
        )


class HourlyHistoryMAP:

    @staticmethod
    def from_domain(location: Location, history: wd.FullHistory) -> List[HourlyHistory]:
        def to_hourly_history(hourly_history: Dict[str, Any]) -> HourlyHistory:
            converter = HistoryConverter(hourly_history)
            return HourlyHistory(
                location_id=location.id,
                time=converter.to_datetime(wd.HourlyWeatherContent.TIME),
                temperature=converter.to_float(wd.HourlyWeatherContent.TEMPERATURE),
                apparent_temperature=converter.to_float(wd.HourlyWeatherContent.APPARENT_TEMPERATURE),
                wind_speed=converter.to_float(wd.HourlyWeatherContent.WIND_SPEED),
                wind_gust=converter.to_float(wd.HourlyWeatherContent.WIND_GUST),
                wind_bearing=converter.to_int(wd.HourlyWeatherContent.WIND_BEARING),
                cloud_cover=converter.to_float(wd.HourlyWeatherContent.CLOUD_COVER),
                humidity=converter.to_float(wd.HourlyWeatherContent.HUMIDITY),
                dew_point=converter.to_float(wd.HourlyWeatherContent.DEW_POINT),
                uv_index=converter.to_int(wd.HourlyWeatherContent.UV_INDEX),
                summary=converter.to_str(wd.HourlyWeatherContent.SUMMARY)
            )

        return [to_hourly_history(hourly) for hourly in history.hourly]
