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
    permissions: Optional[List[str]] = None


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
        for user in self.users:
            user.hashed_password = self.crypt_context.hash(user.hashed_password)

    def get_all(self) -> Generator[WeatherDataUser, None, None]:
        for user in self.users:
            yield user

    def get(self, username: str) -> WeatherDataUser:
        for user in self.users:
            if username == user.username:
                return user.copy(deep=True)

    def verify_password(self, user: WeatherDataUser, password: str) -> bool:
        return self.crypt_context.verify(password, user.hashed_password)
