# The GUI configuration settings.
from typing import Callable

from .weather_settings import SettingName, SettingValue

# the settings singleton
_weather_settings = weather_settings.WeatherSettings()

# the settings API
get_setting: Callable[[SettingName, SettingName], SettingValue] = _weather_settings.get
get_bool_setting: Callable[[SettingName, SettingName], bool] = _weather_settings.get_boolean
get_default_setting: Callable[[SettingName, SettingName], SettingValue] = _weather_settings.get_default
get_int_setting: Callable[[SettingName, SettingName], int] = _weather_settings.get_boolean
get_settings: Callable[[SettingName], SettingValue] = _weather_settings.get_section
set_setting: Callable[[SettingName, SettingName, SettingValue], None] = _weather_settings.set
save_settings: Callable[[], None] = _weather_settings.save
