use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MeasurementDTO {
    pub location_id: i32,
    pub observation_time: DateTime<Utc>,

    // Calculated data
    pub occupancy_rate: Option<i32>,
    pub availability_rate: Option<i32>,
	pub total_vehicles_passed: Option<i32>,
	pub average_speed: Option<i32>,
	pub max_speed: Option<i32>,

	// Location
	pub latitude: f64,
	pub longitude: f64,
}
