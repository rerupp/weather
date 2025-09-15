import argparse
import logging
from sys import stderr

from py_weather_lib import PyWeatherConfig

# this will bootstrap settings
from .config import initialize as config_initialize
from .tk_weather import TkWeather

__all__ = ['run']


def run():
    # parse the command line
    parser = argparse.ArgumentParser(prog="py_gui")
    parser.add_argument('-v', '--verbose', default=0, action='count',
                        help="Log verbosity (default=WARN, v=INFO, vv=DEBUG, vvv+=TRACE).")
    parser.add_argument('-d', '--dir', metavar="DIR", default='weather_dir',
                        help="The weather data directory (default weather_data).")
    parser.add_argument('-l', '--log', metavar="FILE",
                        help="The file where log output will be written.")
    parser.add_argument('-a', '--append', action='store_true',
                        help="When used log output will be appended.")
    options = parser.parse_args()

    # set the log level
    log_level = logging.WARN
    if options.verbose == 1:
        log_level = logging.INFO
    elif options.verbose > 1:
        log_level = logging.DEBUG
    config_initialize(logfile=options.log, log_append=options.append, log_level=log_level)

    # create and run the Tk GUI
    weather_config = PyWeatherConfig(dirname=options.dir, logfile='weather.log', log_level=options.verbose)
    try:
        TkWeather(weather_config).execute()
    except SystemError as e:
        print('System error: %s' % e, file=stderr)
        exit(1)
