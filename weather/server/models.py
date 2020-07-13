from enum import Enum
from typing import Generator, List, Optional

from passlib.context import CryptContext
from pydantic import BaseModel


class Permissions(str, Enum):
    ReadUsers = "users:read"
    ReadWeatherData = "weather_data:read"


class WeatherDataUser(BaseModel):
    username: str
    hashed_password: str
    email: Optional[str] = None
    full_name: Optional[str] = None
    disabled: Optional[bool] = None
    permissions: Optional[List[Permissions]] = None


_weather_data_users = [
    {
        "username": "admin",
        "full_name": "System Administrator",
        "email": "admin@email.com",
        "hashed_password": "admin",
        "disabled": False,
        "permissions": [Permissions.ReadUsers, Permissions.ReadWeatherData]
    },
    {
        "username": "user",
        "full_name": "Weather User",
        "email": "user@email.com",
        "hashed_password": "user",
        "disabled": False,
        "permissions": [Permissions.ReadWeatherData]
    },
    {
        "username": "guest",
        "full_name": "Guest User",
        "email": "guest@email.com",
        "hashed_password": "guest",
        "disabled": True,
        "permissions": [Permissions.ReadWeatherData]
    }
]


class WeatherDataUsers:

    def __init__(self):
        self.crypt_context = CryptContext(schemes=['bcrypt'], deprecated='auto')
        self.users = [WeatherDataUser(**user) for user in _weather_data_users]

    def get_all(self) -> Generator[WeatherDataUser, None, None]:
        for user in self.users:
            yield user

    def get(self, username: str) -> WeatherDataUser:
        for user in self.users:
            if username == user.username:
                return user.copy(deep=True)

    def verify_password(self, user: WeatherDataUser, password: str) -> bool:
        # This is not ideal however there is a pretty big hit hashing the
        # user password (~300 ms/user). For the server that's not a big deal
        # however for other use cases (like in the db package) it make tool
        # startup slow. Since the user implementation is all fake and only
        # called by login, hash the plain text password here instead of in
        # init or implementing a lock scheme. The plain text password could
        # simply be checked here however I wanted to hang onto to how the
        # CryptContext can be used.
        return self.crypt_context.verify(password, self.crypt_context.hash(user.hashed_password))
