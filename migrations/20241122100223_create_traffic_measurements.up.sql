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

-- Create the measurements table which will become a hypertable
CREATE TABLE traffic_measurements (
    location_id INTEGER NOT NULL,

    -- Location data
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    
    -- Timestamps
    observation_time TIMESTAMPTZ NOT NULL,
    
    -- Classification
    vehicle_class vehicle_class NOT NULL,
    
    -- Measurements
    traffic_intensity INTEGER NOT NULL,
    vehicle_speed_arithmetic INTEGER NOT NULL,
    vehicle_speed_harmonic INTEGER NOT NULL,
    
    -- Calculated data
    occupancy_rate DOUBLE PRECISION NOT NULL,
    availability_rate DOUBLE PRECISION NOT NULL,
    instability DOUBLE PRECISION NOT NULL
);

-- Create the hypertable using observation_time
SELECT create_hypertable('traffic_measurements', 'observation_time');

-- Create indices for common queries
CREATE INDEX idx_traffic_measurements_location 
    ON traffic_measurements USING GIST (
        ST_SetSRID(ST_MakePoint(longitude, latitude), 4326)
    );

CREATE INDEX idx_traffic_measurements_metadata 
    ON traffic_measurements(location_id, observation_time DESC);

CREATE INDEX idx_traffic_measurements_vehicle_class 
    ON traffic_measurements(vehicle_class, observation_time DESC);

CREATE UNIQUE INDEX uniq_idx_traffic_measurements_id_time_class
	ON traffic_measurements(location_id, observation_time, vehicle_class);

-- Add compression policy
ALTER TABLE traffic_measurements SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'location_id,vehicle_class'
);

-- Create a continuous aggregate view for hourly statistics
-- CREATE MATERIALIZED VIEW traffic_measurements_hourly
--     WITH (timescaledb.continuous) AS
-- SELECT
--     time_bucket('1 hour', observation_time) AS bucket,
--     descriptive_id,
--     unique_id,
--     vehicle_class,
--     AVG(latitude) as latitude,
--     AVG(longitude) as longitude,
--     AVG(traffic_intensity) AS avg_intensity,
--     AVG(vehicle_speed_arithmetic) AS avg_speed_arithmetic,
--     AVG(vehicle_speed_harmonic) AS avg_speed_harmonic,
--     AVG(occupancy_rate) AS avg_occupancy,
--     AVG(availability_rate) AS avg_availability,
--     COUNT(*) AS measurement_count
-- FROM traffic_measurements
-- GROUP BY bucket, descriptive_id, unique_id, vehicle_class;

-- -- Add refresh policy for the continuous aggregate
-- SELECT add_continuous_aggregate_policy('traffic_measurements_hourly',
--     start_offset => INTERVAL '3 days',
--     end_offset => INTERVAL '1 hour',
--     schedule_interval => INTERVAL '1 hour');

-- Add compression policy after 7 days
SELECT add_compression_policy('traffic_measurements', INTERVAL '7 days');
