use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrafficMeasurement {
    pub location_id: i32,
    pub observation_time: DateTime<Utc>,

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
					observation_time,
					occupancy_rate,
					availability_rate,
					instability
				)
				VALUES ($1, $2, $3, $4, $5)
				ON CONFLICT (location_id, observation_time)
				DO NOTHING
            "#,
            measurement.location_id,
            measurement.observation_time,
            measurement.occupancy_rate,
            measurement.availability_rate,
            measurement.instability,
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
					instability
				) VALUES "
			);

			// Build the values part of the query and collect params
			let values: Vec<String> = batch
				.iter()
				.enumerate()
				.map(|(i, _)| {
					let offset = i * 5;
					format!(
						"(${},${},${},${},${})",
						offset + 1,
						offset + 2,
						offset + 3,
						offset + 4,
						offset + 5
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
					.bind(measurement.instability);
			}

			// Execute the batch insert
			query.execute(pool).await?;
		}

		Ok(())
	}

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
				t.location_id,
				t.observation_time,
				t.occupancy_rate,
				t.availability_rate,
				t.instability
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
            lon, lat, radius, limit
        )
        .fetch_all(pool)
        .await
    }
}
