import json
import shutil
import sys
import zipfile as zf
from contextlib import contextmanager
from datetime import date, datetime, timedelta
from pathlib import Path
from typing import Optional, Tuple, Union, Generator, Callable, List, Set, NamedTuple, Iterator, Dict

import orjson

import weather.errors as err
from weather.configuration import get_logger
from .objects import DataPath, DateRange, Location, WeatherProviderAPI

log = get_logger(__name__)

# typing support
DataContent = Union[str, bytes]
DataSourceReader = Callable[[DataPath], DataContent]
DataSourceWriter = Callable[[DataPath, DataContent], None]
HistoryProperties = List[Tuple[str, dict]]


class WeatherHistoryProperties(NamedTuple):
    entries: int
    entries_size: int
    compressed_size: int
    size: int


class ArchiveDataSource:

    def __init__(self, archive_path: Path):
        self._backup_glob = None
        self._backup_retention = 0
        self._compression = zf.ZIP_DEFLATED
        self._archive_cache: Dict[str, bytes] = {}
        self._archive_path = archive_path
        self._archive_members: Set[str] = set()
        if not archive_path.exists():
            log.warning(f'{archive_path} not found, creating...')
            # opening the archive will create it if it doesn't already exist
            try:
                with self._get_writable_zipfile():
                    pass
            except Exception as exc:
                raise err.WeatherDataError(f'Yikes... Error opening "{str(Path)}": {str(exc)}')
        else:
            try:
                with self._get_readable_zipfile() as zipfile:
                    for name in zipfile.namelist():
                        # verify there are no duplicate histories in the archive
                        if name in self._archive_members:
                            raise err.WeatherDataError(f'Yikes... Found duplicate weather history: {name}')
                        self._archive_members.add(name)
            except err.WeatherDataError:
                pass
            except Exception as exc:
                raise err.WeatherDataError(f'Yikes... Error reading archive name list: {str(exc)}')

    @property
    def compression(self):
        return self._compression

    def clear_cache(self):
        self._archive_cache.clear()

    def data_path_exists(self, data_path: DataPath) -> bool:
        with self._get_readable_zipfile() as zipfile:
            return zf.Path(zipfile, str(data_path)).exists()

    def data_paths(self) -> Generator[DataPath, None, None]:
        with self._get_readable_zipfile() as zipfile:
            for name in zipfile.namelist():
                yield name

    def properties(self) -> WeatherHistoryProperties:
        entries = 0
        entries_size = 0
        compressed_size = 0
        with self._get_readable_zipfile() as zipfile:
            for zip_info in zipfile.infolist():
                entries += 1
                entries_size += zip_info.file_size
                compressed_size += zip_info.compress_size
        return WeatherHistoryProperties(
            entries,
            entries_size,
            compressed_size,
            Path(self._archive_path).stat().st_size
        )

    @contextmanager
    def reader(self) -> DataSourceReader:
        with self._get_readable_zipfile() as zipfile:
            def read_function(data_path: DataPath) -> DataContent:
                data_path = str(data_path) if isinstance(data_path, Path) else data_path
                data_contents = self._archive_cache.get(data_path)
                if not data_contents:
                    with zipfile.open(data_path) as fp:
                        data_contents = fp.read()
                    self._archive_cache[data_path] = data_contents
                return data_contents

            yield read_function

    @contextmanager
    def writer(self) -> DataSourceWriter:

        # the backup archive only exists as long as the writer is active but jic...
        backup_archive: Path = self._archive_path.with_suffix(".bck")
        backup_archive.unlink(missing_ok=True)
        shutil.copy2(str(self._archive_path), str(backup_archive))

        with self._get_writable_zipfile(self._archive_path) as writable_zipfile:
            try:
                def write_function(data_path: DataPath, content: DataContent):
                    data_path = str(data_path) if isinstance(data_path, Path) else data_path
                    if data_path in self._archive_members:
                        raise err.WeatherDataError("'{}' already exists...".format(data_path))
                    zip_info = zf.ZipInfo(data_path, datetime.today().timetuple()[:-3])
                    zip_info.compress_type = zf.ZIP_DEFLATED
                    writable_zipfile.writestr(zip_info, content)
                    self._archive_members.add(data_path)

                yield write_function
            except Exception as error:
                exception = err.WeatherDataError("Yikes!!! Error adding entry: {}".format(error))
                raise exception.with_traceback(sys.exc_info()[2])
            else:
                backup_archive.unlink()
            finally:
                # clean up the backup if it exists at this point, there was an error
                if backup_archive.exists():
                    self._archive_path.unlink()
                    backup_archive.rename(self._archive_path)

    def _get_zipfile(self, path: Path, mode: str) -> zf.ZipFile:
        return zf.ZipFile(path, mode=mode, compression=self.compression)

    def _get_readable_zipfile(self, data_source: Path = None) -> zf.ZipFile:
        return self._get_zipfile(data_source if data_source else self._archive_path, "r")

    def _get_writable_zipfile(self, data_source: Path = None) -> zf.ZipFile:
        data_source = data_source if data_source else self._archive_path
        return self._get_zipfile(data_source, "a" if data_source.exists() else "w")


class DirectoryDataSource:
    """Reads and writes files from a flat directory."""

    def __init__(self, data_path: DataPath, make_if_not_found: bool = True, encoding="UTF-8"):
        self._dir_path = data_path if isinstance(data_path, Path) else Path(data_path)
        self._encoding = encoding
        if not self._dir_path.exists():
            if make_if_not_found:
                self._dir_path.mkdir(parents=True)
            else:
                log.warning("'{}' does not exist...".format(data_path))
        elif not self._dir_path.is_dir():
            raise err.WeatherDataError("Yikes... '{}' is not a directory!".format(self._dir_path))

    @property
    def path(self):
        return self._dir_path

    def get_path(self, data_path: DataPath) -> Path:
        return Path(self._dir_path, data_path)

    def exists(self, data_path: DataPath) -> bool:
        return self.get_path(data_path).exists()

    @contextmanager
    def reader(self) -> DataSourceReader:
        def reader_(data_path: DataPath) -> DataContent:
            full_path = Path(self._dir_path, data_path)
            if not full_path.exists():
                raise err.WeatherDataError("Yikes... '{}' was not found!".format(data_path))
            elif not full_path.is_file():
                raise err.WeatherDataError("Yikes... '{}' is not a data file!".format(data_path))
            with full_path.open('r+b') as fp:
                return fp.read()

        yield reader_

    @contextmanager
    def writer(self) -> DataSourceWriter:
        def writer_(data_path: DataPath, content: DataContent):
            full_path = Path(self._dir_path, data_path)
            if not full_path.parent.exists():
                full_path.parent.mkdir(parents=True)
            with full_path.open('w+b') as fp:
                fp.write(content.encode(self._encoding) if isinstance(content, str) else content)

        yield writer_


MatchLocationCriteria = Union[str, Tuple[str, str], Location]


class Locations:
    DICT_ROOT = "locations"

    def __init__(self, dir_ds: DirectoryDataSource, locations_path: DataPath = Path("locations.json")):
        self._locations: List[Location] = []
        self._dirty: bool = False
        self._dir_ds = dir_ds
        self._locations_path = locations_path if isinstance(locations_path, Path) else Path(locations_path)
        if dir_ds.exists(locations_path):
            with dir_ds.reader() as read_locations:
                locations_dict = json.loads(read_locations(locations_path))
                locations = locations_dict.get(self.DICT_ROOT)
                if not locations:
                    log.warning("Locations root '{}' was not found in '{}'".format(self.DICT_ROOT, locations_path))
                else:
                    for location_dict in locations:
                        location = Location.from_dict(location_dict)
                        if location in self._locations:
                            raise err.WeatherDataError("Duplicate location in '{}': name='{}' alias='{}'"
                                                       .format(self.DICT_ROOT, location.name, location.alias))
                        self._locations.append(location)

    @property
    def is_dirty(self):
        return self._dirty

    # container implementation

    def __delitem__(self, item: Union[str, Location]) -> None:
        idx = self._index_of(item)
        if 0 <= idx:
            self._locations.pop(idx)
            self._dirty = True

    def __getitem__(self, item: MatchLocationCriteria) -> Location:
        idx = self._index_of(item)
        if 0 > idx:
            raise IndexError("'{}' item not found".format(item))
        return self._locations[idx]

    def __len__(self) -> int:
        return len(self._locations)

    def __iter__(self) -> Iterator[Location]:
        for location in self._locations:
            yield location

    def __contains__(self, item: MatchLocationCriteria) -> bool:
        return 0 <= self._index_of(item)

    def _index_of(self, item: MatchLocationCriteria) -> int:
        def get_matcher() -> Callable[[Location], bool]:
            if isinstance(item, str):
                return lambda loc: loc.is_considered(item)
            else:
                name, known_as = (item.name, item.alias) if isinstance(item, Location) else item
                return lambda loc: loc.is_considered(name) or loc.is_considered(known_as)

        item_matches = get_matcher()
        item_idx = -1
        for idx, location in enumerate(self._locations):
            if item_matches(location):
                item_idx = idx
                break
        return item_idx

    def add(self, location: Location):
        if 0 <= self._index_of(location):
            raise err.WeatherDataError("'{}' already exists!!!".format(location.name))
        self._locations.append(location)
        self._dirty = True

    def get(self, name_or_alias: str) -> Optional[Location]:
        idx = self._index_of(name_or_alias)
        return self._locations[idx] if 0 <= idx else None

    def modify(self, location: Location) -> bool:
        idx = self._index_of(location)
        if 0 <= idx:
            self._locations[idx] = location
            self._dirty = True
            return True

    def to_dict(self) -> dict:
        location_dicts: List[dict] = []
        for location in self._locations:
            location_dicts.append(location.to_dict())
        return {self.DICT_ROOT: location_dicts}

    def save(self):
        if self._dirty:
            with self._dir_ds.writer() as write_locations:
                write_locations(self._locations_path, json.dumps(self.to_dict(), indent=2))
            self._dirty = False


class WeatherHistory:

    def __init__(self,
                 archive_path: Path,
                 location: Location,
                 weather_provider: WeatherProviderAPI = WeatherProviderAPI()):
        self._archive_ds = ArchiveDataSource(archive_path)
        self._location = location
        self._weather_provider = weather_provider
        self._dates: Optional[List[date]] = None

    @property
    def location(self) -> Location:
        return self._location

    def clear_cache(self):
        self._archive_ds.clear_cache()

    def add(self, dates: List[date], add_callback: Callable[[date], None]):
        with self.writer() as write:
            for history_date in dates:
                # log.info("getting %s", str(history_date))
                add_callback(history_date)
                history_datetime = datetime.combine(history_date, datetime.min.time())
                response = self._weather_provider.recorded(self.location, history_datetime)
                if WeatherProviderAPI.RECORDED in response:
                    write(self.make_pathname(history_date), json.dumps(response.get(WeatherProviderAPI.RECORDED)))
                    api_calls_today = response.get(WeatherProviderAPI.API_CALLS_MADE)
                    if api_calls_today > WeatherProviderAPI.API_USAGE_LIMIT:
                        log.warning("API call limit exceeded....")
                        break
                elif WeatherProviderAPI.ERROR in response:
                    log.error(response.get(WeatherProviderAPI.ERROR))
                    break
                else:
                    log.critical("Yikes!!! The response did not contain '{}' or '{}'..."
                                 .format(WeatherProviderAPI.RECORDED, WeatherProviderAPI.ERROR))
                    break

    def dates(self, starting_date: date = date.min, ending_date: date = None) -> Optional[List[date]]:
        if not self._dates:
            dates = [self._parse_pathname(dp)[1] for dp in self._archive_ds.data_paths()]
            self._dates = sorted(dates) if len(dates) else None
        if self._dates:
            if not ending_date:
                ending_date = date.max if date.min == starting_date else (starting_date + timedelta(days=1))
            return [history_date for history_date in self._dates if starting_date <= history_date <= ending_date]

    def get_properties(self) -> WeatherHistoryProperties:
        return self._archive_ds.properties()

    def reader(self) -> DataSourceReader:
        return self._archive_ds.reader()

    def writer(self) -> DataSourceWriter:
        return self._archive_ds.writer()

    def make_pathname(self, the_date: date, extension="json") -> str:
        """Creates a name following the form '{location.known_as}'-YYYYMMDD.'{extension}'"""
        prefix = self.location.alias.casefold()
        return "{}/{}-{}.{}".format(prefix, prefix, the_date.isoformat().replace('-', ''), extension)

    # noinspection PyMethodMayBeStatic
    def _parse_pathname(self, path: DataPath) -> Tuple[str, date]:
        """Takes a path whose basename follows the form '{known_as}-YYYYMMDD.json' and
        returns a tuple consisting of location alias and a UTC date created from YYYYMMDD."""
        alias, ascii_date = Path(path).stem.split("-")
        return alias, date(int(ascii_date[0:4]), int(ascii_date[4:6]), int(ascii_date[-2:]))


class WeatherData:
    WEATHER_DATA_DIR = "weather_data"

    def __init__(self, data_dir: DataPath = WEATHER_DATA_DIR, locations_path: DataPath = "locations.json"):
        self._weather_histories: Dict[Location, WeatherHistory] = dict()
        data_dir = data_dir if isinstance(data_dir, Path) else Path(data_dir)
        locations_path = locations_path if isinstance(locations_path, Path) else Path(locations_path)
        log.debug("initializing weather data from {}".format(data_dir.joinpath(locations_path)))
        self._dir_ds = DirectoryDataSource(data_dir)
        self._locations = Locations(self._dir_ds, locations_path)

    def data_path(self) -> Path:
        return self._dir_ds.path

    def close(self):
        self._locations.save()
        weather_histories = self._weather_histories
        for weather_history in weather_histories.values():
            weather_history.clear_cache()
        weather_histories.clear()

    def preload_data(self):
        """The REST services uses this method to cheat multi-thread access to data."""
        for location in self._locations:
            weather_history = self._get_weather_history(location)
            if weather_history:
                with weather_history.reader() as history_reader:
                    for history_date in weather_history.dates():
                        history_reader(weather_history.make_pathname(history_date))

    # locations API

    def locations(self) -> Generator[Location, None, None]:
        for location in self._locations:
            yield location

    def get_location(self, name: str) -> Optional[Location]:
        return self._locations.get(name)

    def add_location(self, location: Location):
        self._locations.add(location)
        self._locations.save()

    def remove_location(self, location: Location) -> bool:
        location = self._locations.get(location.name)
        if location:
            del self._locations[location]
            self._locations.save()
            weather_history = self._get_weather_history(location)
            if weather_history:
                del self._weather_histories[location]
                weather_history.clear_cache()
                self._get_history_path(location).unlink()
            return True

    def update_location(self, name: str, known_as: str) -> bool:
        location = self._locations.get(name)
        if location:
            self._locations.modify(location._replace(alias=known_as.casefold()))
            self._locations.save()
            return True

    # history API

    def add_history(self, location: Location, history_dates: List[date], add_callback: Callable[[date], None]):
        if not history_dates:
            log.warning("There are no history dates to add.")
            return
        self._get_weather_history(location, must_exist=False).add(history_dates, add_callback)
        del self._weather_histories[location]

    def history_exists(self, location: Location) -> bool:
        return self._get_history_path(location).exists()

    def history_dates(self, location: Location, starting: date = None, ending: date = None) -> Optional[List[date]]:
        if self.history_exists(location):
            weather_history = self._get_weather_history(location)
            if starting and ending:
                return weather_history.dates(starting_date=starting, ending_date=ending)
            if starting:
                return weather_history.dates(starting_date=starting)
            if ending:
                return weather_history.dates(ending_date=ending)
            return weather_history.dates()

    def all_history_properties(self) -> List[Tuple[Location, WeatherHistoryProperties]]:
        locations = [location for location in self.locations()]
        locations.sort(key=lambda l: l.name)
        return [(location, self.history_properties(location)) for location in locations]

    def history_properties(self, location: Location) -> WeatherHistoryProperties:
        weather_history = self._get_weather_history(location)
        return weather_history.get_properties() if weather_history else WeatherHistoryProperties(0, 0, 0, 0)

    def history_date_ranges(self, location: Location) -> List[DateRange]:
        history_dates = self.history_dates(location)
        date_ranges: List[DateRange] = []
        if history_dates:
            next_day = timedelta(days=1)
            first = last = history_dates[0]
            for not_used, current in enumerate(history_dates, start=1):
                if current > last + next_day:
                    date_ranges.append(DateRange(first, last))
                    first = last = current
                else:
                    last = current
            date_ranges.append(DateRange(first, last))
        return date_ranges

    def history_path_builder(self, location: Location) -> Optional[Callable[[date], DataPath]]:
        if self.history_exists(location):
            return self._get_weather_history(location).make_pathname

    def get_history(self,
                    location: Location,
                    history_dates: List[date],
                    hourly_history=False) -> Generator[dict, None, None]:
        weather_history = self._get_weather_history(location)
        if not weather_history:
            # return an empty generator
            return
        history_path_builder = self.history_path_builder(location)
        with weather_history.reader() as history_reader:
            for history_path in (history_path_builder(history_date) for history_date in history_dates):
                # history = json.loads(history_reader(history_path))
                history = orjson.loads(history_reader(history_path))
                node = "hourly" if hourly_history else "daily"
                history_node = history.get(node)
                if not history_node:
                    raise err.WeatherDataError("{} history does not contain an '{}' entry..."
                                               .format(location.name, node))
                else:
                    for history_data in history_node.get("data"):
                        yield history_data

    def _get_history_path(self, location: Location) -> Path:
        return Path(self._dir_ds.path, location.alias.casefold()).with_suffix(".zip")

    def _get_weather_history(self, location: Location, must_exist=True) -> Optional[WeatherHistory]:
        weather_history = self._weather_histories.get(location)
        if not weather_history:
            weather_history_path = self._get_history_path(location)
            if must_exist and not weather_history_path.exists():
                pass
            else:
                weather_history = WeatherHistory(weather_history_path, location)
                self._weather_histories[location] = weather_history
        return weather_history
