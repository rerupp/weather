BEGIN;

-- the actual cities table
CREATE TABLE IF NOT EXISTS cities
(
    id        INTEGER PRIMARY KEY,
    city      TEXT NOT NULL COLLATE nocase,
    states_id INTEGER,
    latitude  TEXT NOT NULL,
    longitude TEXT NOT NULL,
    timezone  TEXT NOT NULL,
    FOREIGN KEY (states_id) REFERENCES states (id)
    );
CREATE INDEX IF NOT EXISTS idx_cities_city ON cities (city COLLATE nocase);
CREATE INDEX IF NOT EXISTS idx_cities_states_id ON cities (states_id);

-- consolidate the state names and 2 letter abbreviation to this table
CREATE TABLE IF NOT EXISTS states
(
    id       INTEGER PRIMARY KEY,
    name     TEXT NOT NULL COLLATE nocase,
    state_id TEXT NOT NULL COLLATE nocase
);
CREATE INDEX IF NOT EXISTS idx_states_name on states (name);
CREATE INDEX IF NOT EXISTS idx_states_state_id on states (state_id);

-- the zip code table
CREATE TABLE IF NOT EXISTS zip_codes
(
    id       INTEGER PRIMARY KEY,
    zip_code TEXT UNIQUE
);
CREATE INDEX IF NOT EXISTS idx_zip_codes ON zip_codes (zip_code);

-- create the many-to-many relationship from zip code to city
CREATE TABLE IF NOT EXISTS city_zip_codes
(
    cities_id    INTEGER,
    zip_codes_id INTEGER,
    FOREIGN KEY (cities_id) REFERENCES cities (id),
    FOREIGN KEY (zip_codes_id) REFERENCES zip_codes (id)
    );
CREATE INDEX IF NOT EXISTS idx_city_zip_codes_cities_id on city_zip_codes (cities_id);
CREATE INDEX IF NOT EXISTS idx_city_zip_codes_zip_codes_id on city_zip_codes (zip_codes_id);

-- now commit the batch
COMMIT;
