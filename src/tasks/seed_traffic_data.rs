
use std::env;

use quick_xml::de::from_str;
use sqlx::postgres::PgPoolOptions;

use crate::{errors::AppError, models::{location::Location, traffic_measurement::TrafficMeasurement}, TrafficData, TrafficDataLocations};

pub async fn seed_traffic_data() -> std::result::Result<(), AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").expect("DATABASE_URL missing"))
        .await?;

	let traffic_data_xml = reqwest::get("http://miv.opendata.belfla.be/miv/verkeersdata")
        .await?
        .text()
        .await?;

	let location_data_xml = reqwest::get("http://miv.opendata.belfla.be/miv/configuratie/xml")
        .await?
        .text()
        .await?;

    let traffic_data: TrafficData = from_str(&traffic_data_xml)?;
    let location_data: TrafficDataLocations = from_str(&location_data_xml)?;

	let locations_to_insert = location_data.locations
		.into_iter()
		.map(|location| {
			Location {
				latitude: location.latitude,
				longitude: location.longitude,
				location_id: location.unique_id
			}
		})
		.collect::<Vec<Location>>();
	dbg!(&locations_to_insert.len());
	Location::batch_insert(&pool, locations_to_insert)
		.await?;
	
	const SPECIAL_VALUES: &[i32] = &[251, 252, 254];
	let traffic_measurements_to_insert = traffic_data.measuring_points
		.into_iter()
		.map(|point| {
			let valid_speeds: Vec<i32> = point.measurement_data.iter()
				.map(|m| m.vehicle_speed_arithmetic)
				.filter(|&speed| !SPECIAL_VALUES.contains(&speed))
				.collect();

			let total_vehicles_passed = point.measurement_data.iter()
				.map(|m| m.traffic_intensity)
				.collect::<Vec<i32>>()
				.into_iter()
				.sum::<i32>();

			let average_speed = if !valid_speeds.is_empty() {
				Some((valid_speeds.iter().sum::<i32>() as f64 / valid_speeds.len() as f64).round() as i32)
			} else {
				None
			};

			let max_speed = if !valid_speeds.is_empty() {
				Some(valid_speeds.iter().max().unwrap_or(&0).clone())
			} else {
				None
			};

			TrafficMeasurement {
				location_id: point.unique_id,
				observation_time: point.observation_time.into(),
				occupancy_rate: point.calculated_data.occupancy_rate,
				availability_rate: point.calculated_data.availability_rate,
				total_vehicles_passed,
				average_speed,
				max_speed
				
			}
		})
		.collect::<Vec<TrafficMeasurement>>();
	dbg!(&traffic_measurements_to_insert.len());
	TrafficMeasurement::batch_insert(&pool, traffic_measurements_to_insert)
		.await?;

	Ok(())
}
