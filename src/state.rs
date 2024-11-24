use sqlx::{Pool, Postgres};

#[derive(Clone, Debug)]
pub struct AppState {
	pub pool: Pool<Postgres>,
}
