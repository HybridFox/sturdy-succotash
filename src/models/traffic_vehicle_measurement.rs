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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

	pub async fn batch_insert(
		pool: &sqlx::PgPool,
		measurements: Vec<TrafficVehicleMeasurement>,
	) -> Result<(), sqlx::Error> {
		// Split measurements into batches
		let batches: Vec<Vec<TrafficVehicleMeasurement>> = measurements
			.chunks(1000)
			.map(|chunk| chunk.to_vec())
			.collect();
    
    	// Process each batch
		for batch in batches {
			let mut query_builder = String::from(
				"INSERT INTO public.traffic_vehicle_measurements (
					location_id,
					observation_time,
					vehicle_class,
					traffic_intensity,
					vehicle_speed_arithmetic,
					vehicle_speed_harmonic
				) VALUES "
			);

			// Build the values part of the query and collect params
			let values: Vec<String> = batch
				.iter()
				.enumerate()
				.map(|(i, _)| {
					let offset = i * 6;
					format!(
						"(${},${},${},${},${},${})",
						offset + 1,
						offset + 2,
						offset + 3,
						offset + 4,
						offset + 5,
						offset + 6
					)
				})
				.collect();

			query_builder.push_str(&values.join(","));
			query_builder.push_str(" ON CONFLICT (location_id, observation_time, vehicle_class) DO NOTHING");

			// Build the query
			let mut query = sqlx::query(&query_builder);

			// Add parameters for each measurement
			for measurement in batch {
				query = query
					.bind(measurement.location_id)
					.bind(measurement.observation_time)
					.bind(measurement.vehicle_class as VehicleClass)
					.bind(measurement.traffic_intensity)
					.bind(measurement.vehicle_speed_arithmetic)
					.bind(measurement.vehicle_speed_harmonic);
			}

			// Execute the batch insert
			query.execute(pool).await?;
		}

		Ok(())
	}
}
