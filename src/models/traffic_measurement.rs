use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
