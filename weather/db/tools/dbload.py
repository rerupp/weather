from logging import DEBUG, INFO
from pathlib import Path

import click

from weather import StopWatch
from weather.configuration import get_logger, get_setting, init_logging

_default_db = "weather_data.db"
_weather_data_dir = get_setting("domain", "weather_data_dir")


@click.command("dbload", context_settings=dict(help_option_names=['-h', '--help']))
@click.option('--init/--no-init', 'init', default=True,
              help='Initialize weather data database.', show_default=True)
@click.option('--user/--no-user', 'user', default=True,
              help='Loads the user table.', show_default=True)
@click.option('--loc/--no-loc', 'loc', default=True,
              help='Loads the locations table.', show_default=True)
@click.option('--hist/--no-hist', 'hist', default=True,
              help='Loads the history table.', show_default=True)
@click.option('--norm/--no-norm', 'norm', default=False,
              help='Loads the normalized history tables.', show_default=True)
@click.option('-v', 'verbose', count=True,
              help='Level of messaging from the loader (-v, -vv, etc.).')
@click.option('--db', 'db', type=str, default=_default_db,
              help='The database name.', metavar='DB', show_default=True)
@click.option('--data', 'data_dir', type=str, default=_weather_data_dir,
              help='The weather data directory.', metavar='DIR', show_default=True)
def db_load(init: bool, user: bool, loc: bool, hist: bool, norm: bool, verbose: int, db: str, data_dir: str):
    """
    Runs the database loader. By default the database will be dropped and re-created.
    The DailyHistory and HourlyHistory tables will not be loaded by default because
    they take upwards to a minute to load.
    """
    init_logging()

    parent_package = '.'.join(__package__.split('.')[:-1])
    log = get_logger(parent_package)
    log.setLevel(DEBUG if verbose else INFO)

    db_path = Path(db)
    if db_path.exists():
        if not db_path.is_file():
            log.error(f'{db_path} is not a file...')
            return
        if init:
            db_path.unlink()

    wd_path = Path(data_dir)
    if not (wd_path.exists() and wd_path.is_dir()):
        log.error(f'{wd_path} does not exist or is not a directory...')
        return

    init = StopWatch("Domain and DB init took")
    from weather.domain import WeatherData
    weather_data = WeatherData(wd_path)
    from weather.db import Database
    dialect = 'sqlite:///'
    db_parent = db_path.parent
    url = f'{dialect}./{db_path}' if str(db_parent) == '.' else f'{dialect}{db_path}'
    db = Database(url)
    print(f'{init}.')
    overall = StopWatch("DB load took")
    from weather.db import WeatherDataLoader
    weather_data_loader = WeatherDataLoader(database=db, weather_data=weather_data)
    weather_data_loader.load(users=user, locations=loc, histories=hist, normalized_histories=norm)
    print(f'{overall}.')
