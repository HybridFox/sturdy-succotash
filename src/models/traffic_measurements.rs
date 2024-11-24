use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Debug, Clone, PartialEq, Serialize, Deserialize, Copy)]
#[sqlx(type_name = "vehicle_class", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VehicleClass {
    MotorBikes,
    Cars,
    Vans,
    RigidTrucks,
    ArticulatedTrucks,
    Unknown,
}

impl From<i32> for VehicleClass {
    fn from(value: i32) -> Self {
        match value {
            1 => VehicleClass::MotorBikes,
            2 => VehicleClass::Cars,
            3 => VehicleClass::Vans,
            4 => VehicleClass::RigidTrucks,
            5 => VehicleClass::ArticulatedTrucks,
            _ => VehicleClass::Unknown,
        }
    }
}

impl From<()> for VehicleClass {
    fn from(value: ()) -> Self {
        VehicleClass::Unknown
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrafficMeasurement {
    // Metadata
    pub location_id: i32,

    // Location
    pub latitude: f64,
    pub longitude: f64,

    // Timestamps
    pub observation_time: DateTime<Utc>,

    // Classification
    pub vehicle_class: VehicleClass,

    // Measurements
    pub traffic_intensity: i32,
    pub vehicle_speed_arithmetic: i32,
    pub vehicle_speed_harmonic: i32,

    // Calculated data
    pub occupancy_rate: f64,
    pub availability_rate: f64,
    pub instability: f64,
}

impl TrafficMeasurement {
    pub async fn insert(
        pool: &sqlx::PgPool,
        measurement: TrafficMeasurement,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
				INSERT INTO public.traffic_measurements (
					location_id,
					latitude,
					longitude,
					observation_time,
					vehicle_class,
					traffic_intensity,
					vehicle_speed_arithmetic,
					vehicle_speed_harmonic,
					occupancy_rate,
					availability_rate,
					instability
				)
				VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
				ON CONFLICT (location_id, observation_time, vehicle_class)
				DO NOTHING
            "#,
            measurement.location_id,
            measurement.latitude,
            measurement.longitude,
            measurement.observation_time,
            measurement.vehicle_class as VehicleClass,
            measurement.traffic_intensity,
            measurement.vehicle_speed_arithmetic,
            measurement.vehicle_speed_harmonic,
            measurement.occupancy_rate,
            measurement.availability_rate,
            measurement.instability,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    // Example query to get recent measurements for a specific location
    pub async fn get_recent_by_location(
        pool: &sqlx::PgPool,
        lat: f64,
        lon: f64,
        radius: f64,
        limit: i64,
    ) -> Result<Vec<TrafficMeasurement>, sqlx::Error> {
        sqlx::query_as!(
            TrafficMeasurement,
            r#"
            SELECT
				location_id,
				latitude,
				longitude,
				observation_time,
				traffic_intensity,
				vehicle_speed_arithmetic,
				vehicle_speed_harmonic,
				occupancy_rate,
				availability_rate,
				instability,
				vehicle_class AS "vehicle_class!: VehicleClass"
            FROM public.traffic_measurements
            WHERE ST_DWithin(
                ST_SetSRID(ST_MakePoint(longitude, latitude), 4326),
                ST_SetSRID(ST_MakePoint($1, $2), 4326),
                $3
            )
            ORDER BY observation_time DESC
            LIMIT $4
            "#,
            lon, lat, radius, limit
        )
        .fetch_all(pool)
        .await
    }
}
