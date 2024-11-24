-- Drop the continuous aggregate view
DROP MATERIALIZED VIEW IF EXISTS traffic_measurements_hourly;

-- Drop the main table
DROP TABLE IF EXISTS traffic_measurements;

-- Drop the enum type
DROP TYPE IF EXISTS vehicle_class;
