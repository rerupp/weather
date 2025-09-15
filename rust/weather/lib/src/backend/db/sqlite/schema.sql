BEGIN;

-- The weather locations table
CREATE TABLE IF NOT EXISTS locations
(
    id INTEGER PRIMARY KEY,
    city TEXT NOT NULL COLLATE NOCASE,
    state_id TEXT NOT NULL COLLATE NOCASE,
    state TEXT NOT NULL COLLATE NOCASE,
    name TEXT GENERATED ALWAYS AS (city || ', ' || state_id),
    alias TEXT NOT NULL COLLATE NOCASE,
    longitude TEXT NOT NULL,
    latitude TEXT NOT NULL,
    tz TEXT NOT NULL
);

-- cover the location city name with an index
CREATE INDEX IF NOT EXISTS idx_location_city ON locations(city COLLATE NOCASE);

-- cover the location state id with an index
CREATE INDEX IF NOT EXISTS idx_location_state_id ON locations(state_id COLLATE NOCASE);

-- cover the location state with an index
CREATE INDEX IF NOT EXISTS idx_location_state ON locations(state COLLATE NOCASE);

-- cover the location alias with an index
CREATE UNIQUE INDEX IF NOT EXISTS idx_location_alias ON locations(alias COLLATE NOCASE);

-- cover the location name with an index
CREATE INDEX IF NOT EXISTS idx_location_name ON locations(name COLLATE NOCASE);

-- create the location metadata table
CREATE TABLE if NOT EXISTS metadata
(
    id INTEGER PRIMARY KEY,
    lid INTEGER NOT NULL,
    date TEXT NOT NULL,
    store_size INTEGER,
    size INTEGER,
    FOREIGN KEY (lid) REFERENCES locations(id),
    CONSTRAINT uc_metadata_lid_date UNIQUE (lid, date)
);

-- cover the metadata location id with an index
CREATE INDEX IF NOT EXISTS idx_metadata_lid on metadata(lid);

-- cover the metadata dates with an index
CREATE INDEX IF NOT EXISTS idx_metadata_date on metadata(date);

CREATE TABLE IF NOT EXISTS history
(
    id INTEGER PRIMARY KEY,
    mid INTEGER NOT NULL,
    temp_high REAL,
    temp_low REAL,
    temp_mean REAL,
    dew_point REAL,
    humidity REAL,
    sunrise_t INTEGER,
    sunset_t INTEGER,
    cloud_cover REAL,
    moon_phase REAL,
    uv_index REAL,
    wind_speed REAL,
    wind_gust REAL,
    wind_dir INTEGER,
    visibility REAL,
    pressure REAL,
    precip REAL,
    precip_prob REAL,
    precip_type TEXT,
    description TEXT
);
-- cover the metadata id with an index
CREATE INDEX IF NOT EXISTS idx_history_mid on history(mid);

COMMIT;
