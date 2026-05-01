BEGIN;

-- the country table is the root of other tables
CREATE TABLE IF NOT EXISTS country
(
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL COLLATE nocase,
    code TEXT NOT NULL COLLATE nocase,
    UNIQUE (name, code)
);
CREATE INDEX IF NOT EXISTS idx_country_name on country (name COLLATE nocase);
CREATE INDEX IF NOT EXISTS idx_country_code on country (code COLLATE nocase);

-- the region table identifies distinct areas within a country such as state or province
CREATE TABLE IF NOT EXISTS region
(
    id   INTEGER PRIMARY KEY,
    coid INTEGER,
    name TEXT NOT NULL COLLATE nocase,
    code TEXT NOT NULL COLLATE nocase,
    UNIQUE (coid, name, code),
    -- back link to the country
    FOREIGN KEY (coid) REFERENCES country (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_region_coid on region (coid);
CREATE INDEX IF NOT EXISTS idx_region_name on region (name COLLATE nocase);
CREATE INDEX IF NOT EXISTS idx_region_code on region (code COLLATE nocase);

-- the city metadata
CREATE TABLE IF NOT EXISTS city
(
    id   INTEGER PRIMARY KEY,
    rid  INTEGER,
    name TEXT NOT NULL COLLATE nocase,
    lat  TEXT NOT NULL,
    lng  TEXT NOT NULL,
    tz   TEXT NOT NULL COLLATE nocase,
    -- back link to the region
    FOREIGN KEY (rid) REFERENCES region (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_city_rid ON city (rid);
CREATE INDEX IF NOT EXISTS idx_city_name ON city (name COLLATE nocase);

-- now commit the batch
COMMIT;
