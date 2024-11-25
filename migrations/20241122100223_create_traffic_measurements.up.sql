CREATE EXTENSION IF NOT EXISTS postgis;

-- Create table for locations
CREATE TABLE locations (
    location_id INTEGER PRIMARY KEY,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL
);

-- Create table for measuring point metadata and calculations
CREATE TABLE traffic_measurements (
    -- Primary key and foreign key fields
    location_id INTEGER NOT NULL,
    observation_time TIMESTAMPTZ NOT NULL,

   	availability_rate INTEGER NOT NULL,

    occupancy_rate INTEGER,
    total_vehicles_passed INTEGER,
   	average_speed INTEGER,
   	max_speed INTEGER,
    
    PRIMARY KEY (location_id, observation_time)
);

-- Create hypertable for measuring point data
SELECT create_hypertable('traffic_measurements', 'observation_time');

-- Create indexes for common queries
CREATE INDEX idx_traffic_measurements_location 
    ON traffic_measurements (location_id, observation_time);

-- Create indices for common queries
CREATE INDEX idx_locations_location 
    ON locations USING GIST (
        ST_SetSRID(ST_MakePoint(longitude, latitude), 4326)
    );

CREATE UNIQUE INDEX uniq_idx_traffic_measurements_id_time_class
	ON traffic_measurements(location_id, observation_time);

CREATE UNIQUE INDEX uniq_idx_locations
	ON locations(location_id);
