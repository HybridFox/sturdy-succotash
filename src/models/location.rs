use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub location_id: i32,
    pub latitude: f64,
    pub longitude: f64,
}

impl Location {
    pub async fn insert(
        pool: &sqlx::PgPool,
        location: Location,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
				INSERT INTO public.locations (
					location_id,
					latitude,
					longitude
				)
				VALUES ($1, $2, $3)
				ON CONFLICT (location_id)
				DO NOTHING
            "#,
            location.location_id,
            location.latitude,
            location.longitude
        )
        .execute(pool)
        .await?;

        Ok(())
    }
	
	pub async fn batch_insert(
		pool: &sqlx::PgPool,
		measurements: Vec<Location>,
	) -> Result<(), sqlx::Error> {
		// Split measurements into batches
		let batches: Vec<Vec<Location>> = measurements
			.chunks(1000)
			.map(|chunk| chunk.to_vec())
			.collect();
    
    	// Process each batch
		for batch in batches {
			let mut query_builder = String::from(
				"INSERT INTO public.locations (
					location_id,
					latitude,
					longitude
				) VALUES "
			);

			// Build the values part of the query and collect params
			let values: Vec<String> = batch
				.iter()
				.enumerate()
				.map(|(i, _)| {
					let offset = i * 3;
					format!(
						"(${},${},${})",
						offset + 1,
						offset + 2,
						offset + 3,
					)
				})
				.collect();

			query_builder.push_str(&values.join(","));
			query_builder.push_str(" ON CONFLICT (location_id) DO NOTHING");

			// Build the query
			let mut query = sqlx::query(&query_builder);

			// Add parameters for each measurement
			for measurement in batch {
				query = query
					.bind(measurement.location_id)
					.bind(measurement.latitude)
					.bind(measurement.longitude);
			}

			// Execute the batch insert
			query.execute(pool).await?;
		}

		Ok(())
	}
}
