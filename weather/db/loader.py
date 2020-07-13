import weather.domain as wd
import weather.server.auth as ws
from weather import StopWatch
from weather.configuration import get_logger
from . import database as db, crud

_log = get_logger(__name__)


class WeatherDataLoader:

    def __init__(self, weather_data: wd.WeatherData = wd.WeatherData(), database: db.Database = db.Database()):
        self.weather_data = weather_data
        self.database = database
        self.get_session = database.get_session

    def load(self, users: bool = True, locations: bool = True, histories: bool = True, normalized_histories=True):
        _log.info("Starting database load...")
        self.load_permissions()
        if users:
            self.load_users()
        if locations:
            self.load_locations()
        if histories:
            self.load_histories()
        if normalized_histories:
            self.load_normalized_histories()

    def load_permissions(self):
        with self.get_session() as session:
            # PyCharm is having issues because it groks Permissions as a string not an enum (it's both)
            # noinspection PyTypeChecker
            permissions = crud.add_permissions(session, [p for p in ws.Permissions])
            if permissions:
                nl = "\n  "
                _log.info(f'The following permissions were added{nl}{nl.join([p.name for p in permissions])}')

    def load_users(self):
        with self.get_session() as session:
            users = crud.add_users(session, ws.get_users(), refresh=False)
            if users:
                nl = "\n  "
                _log.info(f'the following users were added{nl}{nl.join([u.username for u in users])}...')

    def load_locations(self):
        with self.get_session() as session:
            locations = crud.add_locations(session, self.weather_data.locations(), refresh=False)
            if locations:
                nl = "\n  "
                _log.info(f'the following locations were added:{nl}{nl.join([l.name for l in locations])}')

    def load_histories(self):
        table_load = StopWatch(label="History Table loaded in")
        for location in self.weather_data.locations():
            dates = self.weather_data.history_dates(location)
            if dates:
                overall = StopWatch("overall:")
                fetch = StopWatch("fetch:")
                histories = [h for h in self.weather_data.get_full_history(location, dates)]
                fetch.stop()
                load = StopWatch("load:")
                with self.get_session() as session:
                    crud.add_histories(session, location, histories, refresh=False)
                _log.info(f'"{location.name}": {fetch} {load} {overall}')
        _log.info(table_load)

    def load_normalized_histories(self):
        """
        This loads history into the DailyHistory and HourlyHistory tables. It isn't being
        used by default because of the weather data domain use case and it is a little
        slow to load data. The History table is taking around 10 seconds while the normalized
        tables are taking around 55 seconds. Still impressive for an interpretive language when
        you consider ~16.4k daily rows and ~390.2k hourly rows compared to time (~7.4k rows/sec).
        """
        table_load = StopWatch("DailyHistory and HourlyHistory tables loaded in")
        for location in self.weather_data.locations():
            dates = self.weather_data.history_dates(location)
            if dates:
                overall = StopWatch("overall:")
                fetch = StopWatch("fetch:")
                histories = [h for h in self.weather_data.get_full_history(location, dates)]
                fetch.stop()
                load = StopWatch("load:")
                with self.get_session() as session:
                    daily = StopWatch("daily:")
                    crud.add_daily_histories(session, location, histories, refresh=False)
                    daily.stop()
                    hourly = StopWatch("hourly:")
                    crud.add_hourly_histories(session, location, histories, refresh=False)
                    hourly.stop()
                _log.info(f'"{location.name}": {fetch} {daily} {hourly} {load} {overall}')
        _log.info(table_load)
