from logging import (DEBUG, INFO, Logger, WARNING, basicConfig, getLogger, ERROR)


class WeatherLog:
    def __init__(self, logfile=None, log_append=False, log_level: int = WARNING, log_format: str = None,
                 date_format: str = None):
        if not log_format:
            log_format = '%(asctime)s %(levelname)s %(module)s: %(message)s'
        if not date_format and 'asctime' in log_format:
            date_format = "%H:%M:%S"
        if not log_level in [ERROR, WARNING, INFO, DEBUG]:
            print(f'Yikes... {log_level} is not ERROR, WARNING, INFO, or DEBUG.')
            log_level = WARNING
        if logfile is None:
            basicConfig(level=log_level, format=log_format, datefmt=date_format)
        else:
            basicConfig(filename=logfile, filemode='a' if log_append else 'w', level=log_level, format=log_format,
                        datefmt=date_format)
        self._root_logger = getLogger('weather')
        self._root_logger.setLevel(log_level)

    def get_logger(self, module_name: str) -> Logger:
        assert module_name, 'A module name is required.'
        # if not self._root_logger:
        #     self.initialize()
        return self._root_logger.getChild(module_name)

    def log_level(self) -> int:
        return self._root_logger.getEffectiveLevel()

    def set_log_level(self, level: int) -> None:
        self._root_logger.setLevel(level)
