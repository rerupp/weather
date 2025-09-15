import sys
from copy import deepcopy
from datetime import datetime
from enum import Enum
from logging import Logger
from pathlib import Path
from typing import Dict, List, Tuple, Union

import importlib_resources
from yaml import MarkedYAMLError, YAMLError, safe_dump, safe_load

from .. import get_logger

USER_SETTINGS_FILENAME = "weather.yaml"
DEFAULT_SETTINGS_RESOURCE = "defaults.yaml"

SettingName = Union[str, Enum]
SettingValue = Union[Dict, List, str, int, bool]


class WeatherSettings:

    def __init__(self):
        self._is_dirty = False
        self._logger = None
        self._default = _load_default_settings()
        self._user = _load_user_settings()

    @staticmethod
    def _get_value(section: dict, name: str) -> SettingValue | None:
        if section:
            for key, value in section.items():
                if key.casefold() == name:
                    return value
        return None

    @staticmethod
    def to_key(name: SettingName) -> str:
        return name.value.casefold() if isinstance(name, Enum) else name.casefold()

    @staticmethod
    def copy(value: SettingValue) -> SettingValue | None:
        if value is not None:
            return deepcopy(value) if isinstance(value, (list, dict)) else value
        return None

    def log(self) -> Logger:
        # you have to be lazy about getting the logger to prevent circular dependencies at startup
        if not hasattr(self, '_logger'):
            self._logger = get_logger(__name__)
        return self._logger

    def _get_section(self, name: str) -> Tuple[dict, dict]:
        default = self._get_value(self._default, name)
        user = self._get_value(self._user, name)
        return default, user

    def _get(self, section: str, name: str) -> SettingValue:
        default, user = self._get_section(section)
        value = self._get_value(user, name)
        if value is None:
            value = self._get_value(default, name)
        return value

    def _get_default(self, section: str, name: str) -> SettingValue | None:
        default, _ = self._get_section(section)
        if default:
            return self._get_value(default, name)
        return None

    def _delete_value(self, section: dict, name: str):
        for key in section.keys():
            if key.casefold() == name:
                del section[key]
                self._is_dirty = True
                break

    def get_section(self, name: SettingName) -> Dict:
        default, user = self._get_section(self.to_key(name))
        combined = deepcopy(default) if default else {}
        if user:
            keys = {k.casefold(): k for k in combined.keys()}
            for key, value in user.items():
                default_key = keys.get(key.casefold())
                key = default_key if default_key else key
                combined[key] = self.copy(value)
        return combined

    def get_default(self, section: SettingName, name: SettingName) -> SettingValue:
        return self.copy(self._get_default(self.to_key(section), self.to_key(name)))

    def get(self, section: SettingName, name: SettingName) -> SettingValue:
        return self.copy(self._get(self.to_key(section), self.to_key(name)))

    def get_int(self, section: SettingName, name: SettingName) -> int:
        value = self._get(self.to_key(section), self.to_key(name))
        if value:
            if isinstance(value, int):
                return value
            if isinstance(value, str):
                try:
                    return int(value)
                except ValueError:
                    return 0
            if isinstance(value, bool):
                return 1 if value else 0
            if isinstance(value, (list, dict)):
                return len(value)
        return 0

    def get_boolean(self, section: SettingName, name: SettingName) -> bool:
        value = self._get(self.to_key(section), self.to_key(name))
        if value:
            if isinstance(value, bool):
                return value
            if isinstance(value, str):
                return value.casefold() in ("y", "yes", "true", "on")
            if isinstance(value, int):
                return 0 != value
            if isinstance(value, (list, dict)):
                return 0 != len(value)
        return False

    def set(self, section: SettingName, name: SettingName, value: SettingValue):
        assert section, "A section is required"
        assert name, "A setting name is required"

        section = self.to_key(section)
        name = self.to_key(name)
        _, user = self._get_section(section)

        # if there is no value consider set a delete operation for the user setting
        if not value:
            if user and self._get_value(user, name) is not None:
                self._delete_value(user, name)
            return

        # if the value matches the default value, delete it
        default_value = self._get_default(section, name)
        if default_value == value:
            if user:
                self._delete_value(user, name)
            return

        # the user configuration may or may not have the configuration section
        if user is None:
            user = dict()
            self._user[section] = user

        # track if the value was set
        is_set = False
        value = self.copy(value)
        # PyCharm sometimes thinks it can't find the keys() method
        # noinspection PyUnresolvedReferences
        for key in user.keys():
            if key.casefold() == name:
                user[key] = value
                is_set = True
                break

        # there's no guarantee the section contained the configuration item
        if not is_set:
            user[name] = value

        self._is_dirty = True

    def save(self):
        if self._is_dirty:
            try:
                yaml_content = "# Saved: {}\n{}".format(datetime.today().strftime("%Y-%m-%d %H:%M:%S"),
                                                        safe_dump(self._user, default_flow_style=False))
                Path(USER_SETTINGS_FILENAME).write_text(yaml_content)
            except YAMLError as err:
                self.log().error("YAMLError: %s", err)
            except OSError as err:
                self.log().error("Error writing '%s': %s", USER_SETTINGS_FILENAME, err)


def _load_default_settings() -> Dict:
    try:
        settings = importlib_resources.files(__package__).joinpath(DEFAULT_SETTINGS_RESOURCE).read_text()
        yaml = safe_load(settings)
        if yaml:
            return yaml
        error = "Warning: default settings were not found!"
    except ImportError as err:
        error = "Yikes... {}!".format(err.msg)
    except FileNotFoundError as err:
        error = "Yikes... {} '{}'!".format(err.strerror, err.filename)
    except MarkedYAMLError as err:
        if err.context:
            error = "{}, {}\n{}".format(err.context, err.problem, err.context_mark)
        else:
            error = "{}\n{}".format(err.context, err.problem_mark)
    except YAMLError as err:
        error = "YAML error: {}}".format(err)
    print(error, file=sys.stderr)
    return dict()


def _load_user_settings() -> Dict:
    user_settings = Path(USER_SETTINGS_FILENAME)
    if user_settings.exists():
        try:
            if not user_settings.is_file():
                error = "'{}' is not a plain file.".format(USER_SETTINGS_FILENAME)
            else:
                settings = user_settings.read_text()
                try:
                    yaml = safe_load(settings)
                    if yaml:
                        return yaml
                    else:
                        error = "Warning: User settings is empty."
                except MarkedYAMLError as err:
                    if err.context:
                        error = "{}, {}\n{}".format(err.context, err.problem, err.context_mark)
                    else:
                        error = "{}\n{}".format(err.problem, err.context_mark)
        except OSError as err:
            error = "Error reading '{}': {}".format(USER_SETTINGS_FILENAME, err)
        # since this is called at startup you need to use stderr and not the log
        print(error, file=sys.stderr)
    return dict()

# def read_user_settings() -> str:
#     user_settings = Path(USER_SETTINGS_FILENAME)
#     try:
#         if user_settings.is_file():
#             return user_settings.read_text()
#         elif user_settings.exists():
#             log.error("'%s' is not a readable file.", user_settings)
#     except OSError as err:
#         log.error("Error reading '%s': %s", user_settings, err)
#
#     # if you're here there was a problem reading the user file so return an empty yaml document
#     return "---\n"
#
#
# def write_user_settings(user_settings: Dict[str, SettingValue]) -> bool:
#     try:
#         yaml_content = "# Saved: {}\n{}".format(datetime.today().strftime("%Y-%m-%d %H:%M:%S"),
#                                                 safe_dump(user_settings, default_flow_style=False))
#         Path(USER_SETTINGS_FILENAME).write_text(yaml_content)
#     except YAMLError as err:
#         log.error("YAMLError: %s", err)
#     except OSError as err:
#         log.error("Error writing '%s': %s", USER_SETTINGS_FILENAME, err)
#     else:
#         return True
