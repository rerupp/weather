import datetime as dt
from typing import Generator, List, Tuple

import pytz
from sqlalchemy import (
    Boolean, Column, Date, DateTime, Float, ForeignKey, Integer, LargeBinary, String, TypeDecorator, UniqueConstraint, or_
)
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import relationship, Session

from weather.configuration import get_logger

_log = get_logger(__name__)
Base = declarative_base()


###############################################################################
# Permissions
###############################################################################


class UserPermissions(Base):
    """Supports the many to many relationship between user and permissions."""
    __tablename__ = "user_permissions"
    user_id = Column(Integer, ForeignKey('users.id'), primary_key=True, index=True)
    permission_id = Column(Integer, ForeignKey('permissions.id'), primary_key=True, index=True)

    def __str__(self):
        return '{}({},{})'.format(
            type(self).__name__,
            f'user_id={self.user_id}',
            f'permission_id={self.permission_id}',
        )


class Permission(Base):
    __tablename__ = "permissions"
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, unique=True)

    def __str__(self):
        return f'{type(self).__name__}(id={self.id},name={self.name})'

    @staticmethod
    def add(session: Session, permissions: List['Permission'], commit=True):
        session.add_all(permissions)
        if commit:
            session.commit()

    @staticmethod
    def exists(session: Session, name: str) -> bool:
        return session.query(Permission).filter(Permission.name == name).count() > 0

    @staticmethod
    def get(session: Session, name: str) -> 'Permission':
        return session.query(Permission).filter(Permission.name == name).first()

    @staticmethod
    def get_all(session: Session) -> Generator['Permission', None, None]:
        for permission in session.query(Permission):
            yield permission


###############################################################################
# Users
###############################################################################


class User(Base):
    __tablename__ = "users"

    id = Column(Integer, primary_key=True, index=True)
    username = Column(String, unique=True, index=True)
    hashed_password = Column(String)
    email = Column(String)
    full_name = Column(String)
    disabled = Column(Boolean, default=True)
    permissions = relationship('Permission', secondary='user_permissions')

    def __str__(self):
        return '{}({},{},{},{},{},{},{})'.format(
            type(self).__name__,
            f'id={self.id}',
            f'username={self.username}',
            f'hashed_password={self.hashed_password}',
            f'email={self.email}',
            f'full_name={self.full_name}',
            f'disabled={self.disabled}',
            f'permissions={self.permissions}'
        )

    @staticmethod
    def add(session: Session, users: List['User'], commit=True):
        # don't use bulk operations because user_permissions will not be populated
        session.add_all(users)
        if commit:
            session.commit()

    @staticmethod
    def exists(session: Session, username: str) -> bool:
        return session.query(User).filter(User.username == username).count() > 0

    @staticmethod
    def get(session: Session, username: str) -> 'User':
        return session.query(User).filter(User.username == username).first()

    @staticmethod
    def get_all(session: Session) -> Generator['User', None, None]:
        for user in session.query(User):
            yield user


###############################################################################
# Locations
###############################################################################


class Location(Base):
    __tablename__ = "locations"

    id = Column(Integer, primary_key=True, index=True)
    full_name = Column(String, unique=True, index=True)
    name = Column(String)
    state = Column(String)
    alias = Column(String, unique=True, index=True)
    longitude = Column(String)
    latitude = Column(String)
    tz = Column(String)

    def __str__(self):
        return '{}({},{},{},{},{},{},{},{})'.format(
            type(self).__name__,
            f'id={self.id}',
            f'full_name={self.full_name}',
            f'name={self.name}',
            f'state={self.state}',
            f'alias={self.alias}',
            f'longitude={self.longitude}',
            f'latitude={self.latitude}',
            f'tz={self.tz}',
        )

    @staticmethod
    def add(session: Session, locations: List['Location'], commit=True):
        session.bulk_save_objects(locations)
        if commit:
            session.commit()

    @staticmethod
    def exists(session: Session, name: str) -> bool:
        return session.query(Location).filter(or_(Location.full_name.like(name), Location.alias.like(name))).count() > 0

    @staticmethod
    def get(session: Session, name: str) -> 'Location':
        return session.query(Location).filter(or_(Location.full_name.like(name), Location.alias.like(name))).first()

    @staticmethod
    def get_all(session: Session, order_by_name: bool = False) -> Generator['Location', None, None]:
        query = session.query(Location)
        if order_by_name:
            query = query.order_by(Location.full_name)
        for location in query:
            yield location


###############################################################################
# History
###############################################################################


class DailyHistory(Base):
    """This table is not being used by the cli."""
    __tablename__ = "daily"

    id = Column(Integer, primary_key=True, index=True)
    location_id = Column(Integer, ForeignKey("locations.id"))
    time = Column(Date, index=True)
    high_temp = Column(Float)
    high_ts = Column(Integer)
    low_temp = Column(Float)
    low_ts = Column(Integer)
    max_temp = Column(Float)
    max_ts = Column(Integer)
    min_temp = Column(Float)
    min_ts = Column(Integer)
    wind_speed = Column(Float)
    wind_gust = Column(Float)
    wind_gust_ts = Column(Integer)
    wind_bearing = Column(Integer)
    cloud_cover = Column(Float)
    humidity = Column(Float)
    dew_point = Column(Float)
    sunrise_ts = Column(Integer)
    sunset_ts = Column(Integer)
    uv_index = Column(Integer)
    uv_index_ts = Column(Integer)
    summary = Column(String)

    __table_args__ = (
        UniqueConstraint('location_id', 'time', name='_daily_history_uc'),
    )

    @staticmethod
    def add(session: Session, histories: List['DailyHistory'], commit=True):
        session.bulk_save_objects(histories)
        if commit:
            session.commit()


# noinspection PyAbstractClass
class TZDateTime(TypeDecorator):
    """This was copied from the SQLAlchemy documentation in the TypeDecorator Recipes section."""
    impl = DateTime

    def process_bind_param(self, value, dialect):
        if value is not None:
            if not value.tzinfo:
                raise TypeError("tzinfo is required")
            value = value.astimezone(pytz.utc).replace(
                tzinfo=None
            )
        return value

    def process_result_value(self, value, dialect):
        if value is not None:
            value = value.replace(tzinfo=pytz.utc)
        return value


class HourlyHistory(Base):
    """This table uses the custom type for storing UTC datetime. It is not currently being used by the cli."""
    __tablename__ = "hourly"

    id = Column(Integer, primary_key=True, index=True)
    location_id = Column(Integer, ForeignKey("locations.id"))
    time = Column(TZDateTime, index=True)
    temperature = Column(Float)
    apparent_temperature = Column(Float)
    wind_speed = Column(Float)
    wind_gust = Column(Float)
    wind_bearing = Column(Integer)
    cloud_cover = Column(Float)
    humidity = Column(Float)
    dew_point = Column(Float)
    uv_index = Column(Integer)
    summary = Column(String)

    __table_args__ = (
        UniqueConstraint('location_id', 'time', name='_hourly_history_uc'),
    )

    @staticmethod
    def add(session: Session, histories: List['HourlyHistory'], commit=True):
        session.bulk_save_objects(histories)
        if commit:
            session.commit()


class History(Base):
    """
    The weather data use case right now is 'Give me hourly or daily history for a date range'.
    There really is no need to have every history attribute visible, you only need to have
    the location and date. The domains understand how to deal with history dictionaries so
    there is no requirement for normalized data.
    """
    __tablename__ = "history"
    id = Column(Integer, primary_key=True, index=True)
    location_id = Column(Integer, ForeignKey("locations.id"))
    date = Column(Date, index=True)
    daily = Column(LargeBinary)
    hourly = Column(LargeBinary)

    __table_args__ = (
        UniqueConstraint('location_id', 'date', name='_history_uc'),
    )

    @staticmethod
    def add(session: Session, histories: List['History'], commit=True):
        session.bulk_save_objects(histories)
        if commit:
            session.commit()

    @staticmethod
    def get_daily(session: Session, location: Location, date_range: Tuple[dt.date, dt.date]) -> Generator['History', None, None]:
        query = session.query(History.id, History.date, History.daily)
        if date_range:
            start, end = date_range
            query = query.filter(History.date >= start, History.date <= end)
        for hid, history_date, daily in query.join(Location).filter(Location.id == location.id):
            yield History(id=hid, date=history_date, daily=daily)

    @staticmethod
    def get_dates(session: Session, location: Location) -> List[dt.date]:
        return [d for d, in session.query(History.date).filter(History.location_id == location.id).order_by(History.date)]
