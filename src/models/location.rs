use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
}
