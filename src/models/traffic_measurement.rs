use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dto::measurement::MeasurementDTO;

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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FindMeasurementsParams {
	pub lat: Option<f64>,
	pub lon: Option<f64>,
	pub radius: f64,
	pub limit: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FindMeasurementsByLocationIdParams {
	pub limit: i64,
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
pub struct TrafficMeasurement {
    pub location_id: i32,
    pub observation_time: DateTime<Utc>,

    // Calculated data
    pub occupancy_rate: i32,
    pub availability_rate: i32,
	
	pub total_vehicles_passed: i32,
	pub average_speed: Option<i32>,
	pub max_speed: Option<i32>,
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
					observation_time,
					occupancy_rate,
					availability_rate,
					total_vehicles_passed,
					average_speed,
					max_speed
				)
				VALUES ($1, $2, $3, $4, $5, $6, $7)
				ON CONFLICT (location_id, observation_time)
				DO NOTHING
            "#,
            measurement.location_id,
            measurement.observation_time,
            measurement.occupancy_rate,
            measurement.availability_rate,
			measurement.total_vehicles_passed,
			measurement.average_speed,
			measurement.max_speed
        )
        .execute(pool)
        .await?;

        Ok(())
    }

	pub async fn batch_insert(
		pool: &sqlx::PgPool,
		measurements: Vec<TrafficMeasurement>,
	) -> Result<(), sqlx::Error> {
		// Split measurements into batches
		let batches: Vec<Vec<TrafficMeasurement>> = measurements
			.chunks(1000)
			.map(|chunk| chunk.to_vec())
			.collect();
    
    	// Process each batch
		for batch in batches {
			let mut query_builder = String::from(
				"INSERT INTO public.traffic_measurements (
					location_id,
					observation_time,
					occupancy_rate,
					availability_rate,
					total_vehicles_passed,
					average_speed,
					max_speed
				) VALUES "
			);

			// Build the values part of the query and collect params
			let values: Vec<String> = batch
				.iter()
				.enumerate()
				.map(|(i, _)| {
					let offset = i * 7;
					format!(
						"(${},${},${},${},${},${},${})",
						offset + 1,
						offset + 2,
						offset + 3,
						offset + 4,
						offset + 5,
						offset + 6,
						offset + 7
					)
				})
				.collect();

			query_builder.push_str(&values.join(","));
			query_builder.push_str(" ON CONFLICT (location_id, observation_time) DO NOTHING");

			// Build the query
			let mut query = sqlx::query(&query_builder);

			// Add parameters for each measurement
			for measurement in batch {
				query = query
					.bind(measurement.location_id)
					.bind(measurement.observation_time)
					.bind(measurement.occupancy_rate)
					.bind(measurement.availability_rate)
					.bind(measurement.total_vehicles_passed)
					.bind(measurement.average_speed)
					.bind(measurement.max_speed);
			}

			// Execute the batch insert
			query.execute(pool).await?;
		}

		Ok(())
	}

    pub async fn get_recent(
        pool: &sqlx::PgPool,
        params: FindMeasurementsParams
	) -> Result<Vec<MeasurementDTO>, sqlx::Error> {
		dbg!(&params);
        sqlx::query_as!(
            MeasurementDTO,
            r#"
            SELECT
				t.location_id,
				t.observation_time,
				t.occupancy_rate,
				t.availability_rate,
				t.total_vehicles_passed,
				t.average_speed,
				t.max_speed,
				l.latitude,
				l.longitude
            FROM public.traffic_measurements t
			LEFT JOIN public.locations l ON t.location_id = l.location_id
            WHERE ST_DWithin(
                ST_SetSRID(ST_MakePoint(l.longitude, l.latitude), 4326),
                ST_SetSRID(ST_MakePoint($1, $2), 4326),
                $3
            )
            ORDER BY observation_time DESC
            LIMIT $4
            "#,
            params.lon, params.lat, params.radius, params.limit
        )
        .fetch_all(pool)
        .await
    }

    pub async fn get_by_location_id(
        pool: &sqlx::PgPool,
		location_id: String,
        params: FindMeasurementsByLocationIdParams
	) -> Result<Vec<MeasurementDTO>, sqlx::Error> {
        sqlx::query_as!(
            MeasurementDTO,
            r#"
            SELECT
				t.location_id,
				t.observation_time,
				t.occupancy_rate,
				t.availability_rate,
				t.total_vehicles_passed,
				t.average_speed,
				t.max_speed,
				l.latitude,
				l.longitude
            FROM public.traffic_measurements t
			LEFT JOIN public.locations l ON t.location_id = l.location_id
            WHERE l.location_id = $1
            ORDER BY observation_time DESC
            LIMIT $2
            "#,
            location_id.parse().unwrap_or(0), params.limit
        )
        .fetch_all(pool)
        .await
    }
}
