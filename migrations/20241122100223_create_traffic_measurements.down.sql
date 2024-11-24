-- Drop the continuous aggregate view
DROP MATERIALIZED VIEW IF EXISTS traffic_measurements_hourly;

-- Drop the main table
DROP TABLE IF EXISTS traffic_measurements;
DROP TABLE IF EXISTS traffic_vehicle_measurements;
DROP TABLE IF EXISTS locations;

-- Drop the enum type
DROP TYPE IF EXISTS vehicle_class;
