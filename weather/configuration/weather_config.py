from copy import deepcopy
from datetime import datetime
from enum import Enum
from importlib.resources import read_text
from logging import basicConfig, Formatter, getLogger, Logger, LogRecord, StreamHandler, DEBUG, INFO, WARNING
from pathlib import Path
from typing import Callable, Dict, List, Tuple, Union, NamedTuple

from yaml import safe_load, safe_dump, YAMLError, MarkedYAMLError

log = getLogger(__name__)

SettingName = Union[str, Enum]
SettingValue = Union[Dict, List, str, int, bool]


class WeatherSettings:
    _is_dirty = False

    def __init__(self):
        self._default = load_yaml(read_package_data("defaults.yaml"))
        self._user = load_yaml(read_user_settings())

    @staticmethod
    def _get_value(section: dict, name: str) -> SettingValue:
        if section:
            for key, value in section.items():
                if key.casefold() == name:
                    return value

    @staticmethod
    def to_key(name: SettingName) -> str:
        return name.value.casefold() if isinstance(name, Enum) else name.casefold()

    @staticmethod
    def copy(value: SettingValue) -> SettingValue:
        if value is not None:
            return deepcopy(value) if isinstance(value, (list, dict)) else value

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

    def _get_default(self, section: str, name: str) -> SettingValue:
        default, _ = self._get_section(section)
        if default:
            return self._get_value(default, name)

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

    def save(self) -> bool:
        if not self._is_dirty:
            return True
        if write_user_settings(self._user):
            self._is_dirty = False
            return True


def load_yaml(yaml_str: str) -> Dict:
    try:
        yaml = safe_load(yaml_str)
        return yaml if yaml else dict()
    except MarkedYAMLError as yaml_err:
        if yaml_err.context:
            _yaml_errmsg = "{}, {}\n{}".format(yaml_err.context, yaml_err.problem, yaml_err.context_mark)
        else:
            _yaml_errmsg = "{}\n{}".format(yaml_err.context, yaml_err.problem_mark)
        log.error("YAML: %s", _yaml_errmsg)
    except YAMLError as yaml_err:
        log.error("YAML error: %s", yaml_err)

    # if you fall out to here an exception was thrown so ensure a dict is returned
    return dict()


def read_package_data(name: str) -> str:
    try:
        return read_text("{}.data".format(__package__), name)
    except ImportError as error:
        # PyCharm is having issues groking the error attributes
        # noinspection PyUnresolvedReferences
        log.error("Yikes... {}!".format(error.msg))
    except FileNotFoundError as error:
        log.error("Yikes... {} '{}'!".format(error.strerror, error.filename))

    # if you're here there was a problem reading the package file so return an empty yaml document
    return "---\n"


USER_SETTINGS_FILENAME = "weather.yaml"


def read_user_settings() -> str:
    user_settings = Path(USER_SETTINGS_FILENAME)
    try:
        if user_settings.is_file():
            return user_settings.read_text()
        elif user_settings.exists():
            log.error("'%s' is not a readable file.", user_settings)
    except OSError as err:
        log.error("Error reading '%s': %s", user_settings, err)

    # if you're here there was a problem reading the user file so return an empty yaml document
    return "---\n"


def write_user_settings(user_settings: Dict[str, SettingValue]) -> bool:
    try:
        yaml_content = "# Saved: {}\n{}".format(datetime.today().strftime("%Y-%m-%d %H:%M:%S"),
                                                safe_dump(user_settings, default_flow_style=False))
        Path(USER_SETTINGS_FILENAME).write_text(yaml_content)
    except YAMLError as err:
        log.error("YAMLError: %s", err)
    except OSError as err:
        log.error("Error writing '%s': %s", USER_SETTINGS_FILENAME, err)
    else:
        return True


_weather_settings = WeatherSettings()
get_setting: Callable[[SettingName, SettingName], SettingValue] = _weather_settings.get
get_bool_setting: Callable[[SettingName, SettingName], bool] = _weather_settings.get_boolean
get_default_setting: Callable[[SettingName, SettingName], SettingValue] = _weather_settings.get_default
get_int_setting: Callable[[SettingName, SettingName], int] = _weather_settings.get_boolean
get_settings: Callable[[SettingName], SettingValue] = _weather_settings.get_section
set_setting: Callable[[SettingName, SettingName, SettingValue], None] = _weather_settings.set
save_settings: Callable[[], bool] = _weather_settings.save


######################################################################################
#
# GUI colors
#
######################################################################################


class Color(NamedTuple):
    name: str
    red: int
    green: int
    blue: int

    def to_hex(self):
        return "#{:02x}{:02x}{:02x}".format(self.red, self.green, self.blue)


def load_rgb_txt() -> Tuple[Color, ...]:
    content = []
    for line in read_package_data("rgb.txt").splitlines():
        if not line.startswith("!"):
            red, green, blue, name = line.split(maxsplit=3)
            content.append(Color(name, int(red), int(green), int(blue)))

    # this should only happen if there was an error reading the data package file
    if not content:
        # return the colors tkinter guarantees
        content = [
            Color("white", 255, 255, 255),
            Color("black", 0, 0, 0),
            Color("red", 255, 0, 0),
            Color("green", 0, 255, 0),
            Color("blue", 0, 0, 255),
            Color("cyan", 0, 255, 255),
            Color("yellow", 255, 255, 0),
            Color("magenta", 255, 0, 255)
        ]
    return tuple(content)


_rgb = None


def get_colors() -> Tuple[Color, ...]:
    global _rgb
    if not _rgb:
        _rgb = load_rgb_txt()
    return _rgb


######################################################################################
#
# logging support
#
######################################################################################


_logger_module_names: List[str] = []


def get_logger(module_name: str) -> Logger:
    assert module_name, 'The module_name is required...'
    logger = getLogger(module_name)
    if module_name not in _logger_module_names:
        _logger_module_names.append(module_name)
    set_logging_level(logger, module_name)
    return logger


def set_module_logging_levels():
    for module_name in _logger_module_names:
        set_logging_level(getLogger(module_name), module_name)


def get_log_level(logging_level: str) -> int:
    level = logging_level.casefold()
    return INFO if "info" == level else DEBUG if "debug" == level else WARNING


def set_logging_level(logger: Logger, configuration_name: str):
    # walk the module hierarchy looking for a configuration setting
    hierarchy = configuration_name.split('.')
    hierarchy.reverse()
    for configuration_name in hierarchy:
        logging_level = get_setting(configuration_name, "logging_level")
        if logging_level:
            logger.setLevel(get_log_level(logging_level))
            return


class WeatherLogFormatter(Formatter):
    def format(self, record: LogRecord) -> str:
        if record.levelno == INFO:
            return record.getMessage()
        else:
            return super(WeatherLogFormatter, self).format(record)


def init_logging():
    handler = StreamHandler()
    log_format = _weather_settings.get("logging", "log_format")
    if not log_format:
        log_format = "%(levelname)s: %(message)s"
    handler.setFormatter(WeatherLogFormatter(fmt=log_format))

    level = get_log_level(get_setting("logging", "default_logging_level"))
    # noinspection PyArgumentList
    basicConfig(level=level, handlers=[handler])
