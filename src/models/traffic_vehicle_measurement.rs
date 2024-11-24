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

#[derive(Debug, Serialize, Deserialize)]
pub struct TrafficVehicleMeasurement {
    pub location_id: i32,
    pub observation_time: DateTime<Utc>,
	pub vehicle_class: VehicleClass,

    // Calculated data
    pub traffic_intensity: i32,
    pub vehicle_speed_arithmetic: i32,
    pub vehicle_speed_harmonic: i32,
}

impl TrafficVehicleMeasurement {
    pub async fn insert(
        pool: &sqlx::PgPool,
        measurement: TrafficVehicleMeasurement,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
				INSERT INTO public.traffic_vehicle_measurements (
					location_id,
					observation_time,
					vehicle_class,
					traffic_intensity,
					vehicle_speed_arithmetic,
					vehicle_speed_harmonic
				)
				VALUES ($1, $2, $3, $4, $5, $6)
				ON CONFLICT (location_id, observation_time, vehicle_class)
				DO NOTHING
            "#,
            measurement.location_id,
            measurement.observation_time,
            measurement.vehicle_class as VehicleClass,
            measurement.traffic_intensity,
            measurement.vehicle_speed_arithmetic,
			measurement.vehicle_speed_harmonic
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
