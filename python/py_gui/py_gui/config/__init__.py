# The GUI configuration package
from logging import Logger, WARN
from typing import Optional

from .weather_log import WeatherLog

__all__ = ['initialize', 'get_logger']

__weather_log: Optional[WeatherLog] = None


def initialize(logfile: Optional[str] = None, log_append=False, log_level=WARN):
    global __weather_log
    __weather_log = WeatherLog(logfile=logfile, log_append=log_append, log_level=log_level)


def get_logger(module_name: str) -> Logger:
    if not __weather_log:
        initialize()
    return __weather_log.get_logger(module_name)
