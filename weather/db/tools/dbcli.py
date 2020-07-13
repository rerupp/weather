from collections import namedtuple
from datetime import date, datetime, timedelta
from logging import DEBUG, INFO
from pathlib import Path
from typing import Callable, Dict, Iterable, List, Tuple, TypeVar

import click
import pytz
from sqlalchemy.orm import Session

import weather.db as db
import weather.domain as wd
import weather.server as srvr
from weather import StopWatch
from weather.configuration import get_logger, get_setting, init_logging

_log = get_logger(__package__)
_default_db = "weather_data.db"
_weather_data_dir = get_setting("domain", "weather_data_dir")


@click.group("dbcli", context_settings=dict(help_option_names=['-h', '--help']))
@click.option('-v', 'verbose', count=True,
              help='Level of messaging from the cli (-v, -vv, etc.).')
@click.option('--db', 'db_name', type=str, default=_default_db,
              help='The database name.', metavar='DB', show_default=True)
@click.pass_context
def cli(ctx: click.Context, verbose: int, db_name: str):
    init_logging()
    _log.setLevel(DEBUG if verbose else INFO)
    if verbose > 1:
        parent_package = '.'.join(__package__.split('.')[:-1])
        log = get_logger(parent_package)
        log.setLevel(DEBUG if verbose else INFO)

    db_path = Path(db_name)
    if not (db_path.exists() and db_path.is_file()):
        raise click.BadOptionUsage(option_name='--db', message=f'"{db}" is not a file or does not exist...')
    dialect = 'sqlite:///'
    db_parent = db_path.parent
    url = f'{dialect}./{db_path}' if str(db_parent) == '.' else f'{dialect}{db_path}'
    ctx.obj = db.Database(url)


T = TypeVar('T')


def max_width(items: Iterable[T], min_width: int = 0, get: Callable[[T], str] = lambda t: t):
    return max(min_width, max((len(get(item)) for item in items), default=0))


@cli.command("lu")
@click.argument('user_filter', nargs=-1, metavar='[USER]...')
@click.pass_obj
def list_user(database: db.Database, user_filter: tuple):
    """List the weather data users defined for the REST service API."""
    overall = StopWatch("lu overall time", in_ms=True)
    # there aren't that many users so just iterate over the results and see if there is a
    # match
    users: List[srvr.WeatherDataUser] = []
    with database.get_session() as session:
        for user in db.get_users(session):
            if user_filter and user.username not in user_filter:
                continue
            users.append(user)
    if not users:
        _log.warning(f'No users {"matched selection" if user_filter else "were found"}.')
    else:
        Field = namedtuple('Field', ['key', 'header', 'detail', 'title'])
        fields: List[Field] = [
            Field("un", "{:-^{un}}", "{:<{un}}", 'Username'),
            Field("fn", "{:-^{fn}}", "{:<{fn}}", 'Full Name'),
            Field("em", "{:-^{em}}", "{:<{em}}", 'Email'),
            Field("df", "{:-^{df}}", "{:^{df}}", 'Disabled'),
            Field("pm", "{:-^{pm}}", "{:<{pm}}", 'Permissions'),
        ]
        header = " ".join([f.header for f in fields])
        detail = " ".join([f.detail for f in fields])
        field_widths: Dict[str, int] = {
            fields[0].key: max_width(users, min_width=len(fields[0].title), get=lambda u: u.username),
            fields[1].key: max_width(users, min_width=len(fields[1].title), get=lambda u: u.full_name),
            fields[2].key: max_width(users, min_width=len(fields[2].title), get=lambda u: u.email),
            fields[3].key: len(fields[3].title),
            fields[4].key: 2 * len(fields[4].title),
        }
        print(header.format(*[f.title for f in fields], **field_widths))
        for user in users:
            disabled = "Yes" if user.disabled else ""
            permissions = " ".join(p.value for p in user.permissions)
            print(detail.format(user.username, user.full_name, user.email, disabled, permissions,
                                **field_widths))
    _log.debug(f'{overall}.')


def _get_locations(session: Session, locations_filter: tuple) -> List[wd.Location]:
    locations: List[wd.Location] = []
    if not locations_filter:
        locations.extend([location for location in db.get_locations(session)])
    else:
        for name in locations_filter:
            location = db.get_location(session, name)
            if not location:
                _log.warning(f'Location "{name}" not found.')
                continue
            locations.append(location)
    return locations


@cli.command("ll")
@click.argument('locations_filter', nargs=-1, metavar='[LOC]...')
@click.pass_obj
def list_locations(database: db.Database, locations_filter: tuple):
    """List the weather data locations."""
    overall = StopWatch("ll overall time", in_ms=True)
    with database.get_session() as session:
        locations: List[wd.Location] = _get_locations(session, locations_filter)
    if not locations:
        _log.warning(f'No locations {"matched selection" if locations_filter else "were found"}.')
    else:
        Field = namedtuple('Field', ['key', 'header', 'detail', 'title'])
        fields: List[Field] = [
            Field("ln", "{:-^{ln}}", "{:<{ln}}", 'Location'),
            Field("an", "{:-^{an}}", "{:<{an}}", 'Alias'),
            Field("lg", "{:->{lg}}", "{:>{lg}}", 'Longitude'),
            Field("lt", "{:-<{lt}}", "{:<{lt}}", 'Latitude'),
            Field("tz", "{:-^{tz}}", "{:<{tz}}", 'Timezone'),
        ]
        header = f'{fields[0].header} {fields[1].header} {fields[2].header}/{fields[3].header} {fields[4].header}'
        detail = f'{fields[0].detail} {fields[1].detail} {fields[2].detail}/{fields[3].detail} {fields[4].detail}'
        field_widths: Dict[str, int] = {
            fields[0].key: max_width(locations, min_width=len(fields[0].title), get=lambda l: l.name),
            fields[1].key: max_width(locations, min_width=len(fields[1].title), get=lambda l: l.alias),
            fields[2].key: 12,
            fields[3].key: 12,
            fields[4].key: max_width(locations, min_width=len(fields[4].title), get=lambda l: l.tz),
        }
        print(header.format(*[f.title for f in fields], **field_widths))
        for location in locations:
            print(detail.format(location.name, location.alias, location.longitude, location.latitude, location.tz,
                                **field_widths))
    _log.debug(f'{overall}.')


@cli.command("lh")
@click.argument('locations_filter', nargs=-1, metavar='[LOC]...')
@click.pass_obj
def list_history(database: db.Database, locations_filter: tuple):
    """List what history is available for the weather data locations."""
    overall = StopWatch("ll overall time", in_ms=True)
    with database.get_session() as session:
        location_histories: List[Tuple[str, List[date]]] = [
            (location.name, db.get_history_dates(session, location))
            for location in _get_locations(session, locations_filter)
        ]

    if not location_histories:
        _log.warning(f'No locations {"matched selection" if locations_filter else "were found"}.')
    else:
        location_date_ranges: List[Tuple[str, List[wd.DateRange]]] = []
        next_day = timedelta(days=1)
        for location, history_dates in location_histories:
            date_ranges: List[wd.DateRange] = []
            if history_dates:
                first = last = history_dates[0]
                for _, current in enumerate(history_dates, start=1):
                    if current > last + next_day:
                        date_ranges.append(wd.DateRange(first, last))
                        first = last = current
                    else:
                        last = current
                date_ranges.append(wd.DateRange(first, last))
            location_date_ranges.append((location, date_ranges))
        Field = namedtuple('Field', ['key', 'header', 'detail', 'title'])
        fields: List[Field] = [
            Field("ln", "{:-^{ln}}", "{:<{ln}}", 'Location'),
            Field("hd", "{:-^{hd}}", "{:<{hd}}", 'History Dates'),
        ]
        header = " ".join(f.header for f in fields)
        detail = " ".join(f.detail for f in fields)
        field_widths: Dict[str, int] = {
            fields[0].key: max_width(location_date_ranges, min_width=len(fields[0].title), get=lambda ldr: ldr[0]),
            fields[1].key: len("XXXX-XX-XX to XXXX-XX-XX"),
        }
        print(header.format(*[f.title for f in fields], **field_widths))
        for location, date_ranges in location_date_ranges:
            if not date_ranges:
                print(detail.format(location, "None", **field_widths))
            else:
                for start, end in date_ranges:
                    print(detail.format(location, f'{start} to {end}' if start and end else str(start), **field_widths))
                    location = ""
    _log.debug(f'{overall}.')


@cli.command("rh")
@click.option('-s', '--start', 'start', type=click.DateTime(formats=["%Y-%m-%d"]), metavar='START',
              help="The history start date (ISO format).")
@click.option('-e', '--end', 'end', nargs=1, type=click.DateTime(formats=["%Y-%m-%d"]), metavar='END',
              help="The history end date (ISO format).")
@click.argument('location_name', nargs=1, metavar='LOCATION')
@click.pass_obj
def report_history(database: db.Database, location_name: str, start: datetime, end: datetime):
    """Get history from a locations weather data. LOCATION can be either the location name or alias name."""
    overall = StopWatch("rh overall time", in_ms=True)
    if start:
        start = start.date()
    if end:
        end = end.date()
    date_range = (wd.DateRange(low=start if start else end, high=end if end else start)) if start or end else None
    histories: List[wd.FullHistory] = []
    with database.get_session() as session:
        location = db.get_location(session, location_name)
        if location:
            for history in db.get_daily_history(session, location, date_range):
                histories.append(history)
            location_tz = location.tz
    if not histories:
        _log.warning(f'No history was found for "{location_name}".')
    else:
        Field = namedtuple('Field', ['key', 'header', 'detail', 'titles'])
        fields: List[Field] = [
            Field("dt", "{:^{dt}}", "{:^{dt}}", ("", "Date")),
            Field("ht", "{:^{ht}}", "{:^{ht}}", ("High", "Temperature")),
            Field("htd", "{:^{htd}}", "{:^{htd}}", ("High", "Temperature TOD")),
            Field("lt", "{:^{lt}}", "{:^{lt}}", ("Low", "Temperature")),
            Field("ltd", "{:^{ltd}}", "{:^{ltd}}", ("Low", "Temperature TOD")),
        ]
        header = " ".join(field.header for field in fields)
        detail = " ".join(field.detail for field in fields)
        field_widths: Dict[str, int] = {
            fields[0].key: len(str(date.today())),
            fields[1].key: len(fields[1].titles[1]),
            fields[2].key: len(fields[2].titles[1]),
            fields[3].key: len(fields[3].titles[1]),
            fields[4].key: len(fields[4].titles[1]),
        }
        print(header.format(*[field.titles[0] for field in fields], **field_widths))
        print(header.format(*[field.titles[1] for field in fields], **field_widths))
        print(header.format(*("-" * field_widths[field.key] for field in fields), **field_widths))

        def to_fahrenheit(data):
            return wd.DataConverter.to_fahrenheit(data) if data else ""

        def to_time(data):
            return wd.DataConverter.to_time(data, tz, fmt="%H:%M") if data else ""

        tz = pytz.timezone(location_tz)
        for history_date, daily, _ in histories:
            print(detail.format(
                str(history_date),
                to_fahrenheit(daily.get(wd.DailyWeatherContent.TEMPERATURE_HIGH.value)),
                to_time(daily.get(wd.DailyWeatherContent.TEMPERATURE_HIGH_TIME.value)),
                to_fahrenheit(daily.get(wd.DailyWeatherContent.TEMPERATURE_LOW.value)),
                to_time(daily.get(wd.DailyWeatherContent.TEMPERATURE_LOW_TIME.value)),
                **field_widths
            ))
    _log.debug(f'{overall}.')


# noinspection SqlNoDataSourceInspection
@cli.command("ls")
@click.pass_obj
def list_stats(database: db.Database):
    """Get information about the weather data database."""
    if not database.is_sqlite:
        _log.warning(f'ls is not implemented for {database.db_url[:database.db_url.find("://")]}.')
    else:
        overall = StopWatch("ls overall time", in_ms=True)
        with database.engine.connect() as connection:
            sql = "select sum(pgsize-unused) from dbstat where name = '{}';"
            table_metrics = [(table, connection.execute(sql.format(table)).scalar())
                             for table in ('daily', 'hourly', 'history')]
            location_metrics = [(name, histories)
                                for name, histories in connection.execute("""
            select locations.full_name, count(*) from locations
            join history on locations.id = history.location_id
            group by locations.full_name order by locations.full_name;
            """)]
        Field = namedtuple('Field', ['key', 'header', 'detail', 'title'])
        fields: List[Field] = [
            Field("ln", "{:^{ln}}", "{:<{ln}}", "Location"),
            Field("hc", "{:^{hc}}", "{:>{hc}}", "Histories"),
            Field("dt", "{:^{dt}}", "{:>{dt}}", "Daily Table"),
            Field("hr", "{:^{hr}}", "{:>{hr}}", "Hourly Table"),
            Field("ht", "{:^{ht}}", "{:>{ht}}", "History Table"),
            Field("os", "{:^{os}}", "{:>{os}}", "Tables Combined"),
        ]
        header = " ".join(field.header for field in fields)
        detail = " ".join(field.detail for field in fields)
        field_widths: Dict[str, int] = {
            fields[0].key: max_width(location_metrics, min_width=len(fields[0].title), get=lambda lm: lm[0]),
            fields[1].key: len(fields[1].title),
            fields[2].key: len(fields[2].title),
            fields[3].key: len(fields[3].title),
            fields[4].key: len(fields[4].title),
            fields[5].key: len(fields[5].title),
        }
        print(header.format(*[field.title for field in fields], **field_widths))
        print(header.format(*("-" * field_widths[field.key] for field in fields), **field_widths))

        def fmt(value: int, kib=True) -> str:
            return "" if not value else f'{value: >,d}' if not kib or value < 1024 else f'{round(value / 1024): >,d} kiB'

        total_histories = sum(c for _, c in location_metrics)
        daily_size = next((size for name, size in table_metrics if name == 'daily'), 0)
        hourly_size = next((size for name, size in table_metrics if name == 'hourly'), 0)
        history_size = next((size for name, size in table_metrics if name == 'history'), 0)
        total_size = sum(b for _, b in table_metrics)
        for location, history_count in location_metrics:
            percent_total = history_count / total_histories
            print(detail.format(location,
                                fmt(history_count, kib=False),
                                fmt(daily_size * percent_total),
                                fmt(hourly_size * percent_total),
                                fmt(history_size * percent_total),
                                fmt(total_size * percent_total),
                                **field_widths))
        print(header.format(*("=" * field_widths[field.key] for field in fields), **field_widths))
        print(detail.format("Totals",
                            fmt(total_histories, kib=False),
                            fmt(daily_size),
                            fmt(hourly_size),
                            fmt(history_size),
                            fmt(total_size),
                            **field_widths))
        _log.debug(f'{overall}.')
