CREATE EXTENSION IF NOT EXISTS postgis;

-- First create an enum for vehicle classes
CREATE TYPE vehicle_class AS ENUM (
    'MOTOR_BIKES',
    'CARS',
    'VANS',
    'RIGID_TRUCKS',
    'ARTICULATED_TRUCKS',
    'UNKNOWN'
);

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
    
    -- Calculated data (shared across vehicle classes)
    occupancy_rate DOUBLE PRECISION NOT NULL,
    availability_rate DOUBLE PRECISION NOT NULL,
    instability DOUBLE PRECISION NOT NULL,
    
    PRIMARY KEY (location_id, observation_time),
    FOREIGN KEY (location_id) REFERENCES locations (location_id)
);

-- Create hypertable for measuring point data
SELECT create_hypertable('traffic_measurements', 'observation_time');

-- Create table for vehicle class specific measurements
CREATE TABLE traffic_vehicle_measurements (
    -- Primary key and foreign key fields
    location_id INTEGER NOT NULL,
    observation_time TIMESTAMPTZ NOT NULL,
    vehicle_class vehicle_class NOT NULL,
    
    -- Measurements
    traffic_intensity INTEGER NOT NULL,
    vehicle_speed_arithmetic INTEGER NOT NULL,
    vehicle_speed_harmonic INTEGER NOT NULL,
    
    PRIMARY KEY (location_id, observation_time, vehicle_class)
);

-- Create hypertable for vehicle measurements
SELECT create_hypertable('traffic_vehicle_measurements', 'observation_time');

-- Create indexes for common queries
CREATE INDEX idx_traffic_measurements_location 
    ON traffic_measurements (location_id, observation_time);
    
CREATE INDEX idx_traffic_vehicle_measurements_lookup 
    ON traffic_vehicle_measurements (location_id, observation_time, vehicle_class);

-- If you're using PostGIS, you might want to add:
ALTER TABLE locations ADD COLUMN geom geometry(Point, 4326);
CREATE INDEX idx_locations_geom ON locations USING GIST (geom);

-- Create indices for common queries
CREATE INDEX idx_locations_location 
    ON locations USING GIST (
        ST_SetSRID(ST_MakePoint(longitude, latitude), 4326)
    );

CREATE UNIQUE INDEX uniq_idx_traffic_measurements_id_time_class
	ON traffic_measurements(location_id, observation_time);

CREATE UNIQUE INDEX uniq_idx_locations
	ON locations(location_id);

CREATE UNIQUE INDEX uniq_idx_traffic_vehicle_measurements
	ON traffic_vehicle_measurements(location_id, observation_time, vehicle_class);
