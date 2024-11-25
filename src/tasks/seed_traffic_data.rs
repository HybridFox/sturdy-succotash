
use std::env;

use futures::StreamExt;
use quick_xml::de::from_str;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::{errors::AppError, models::{location::Location, traffic_measurement::TrafficMeasurement, traffic_vehicle_measurement::TrafficVehicleMeasurement}, TrafficData, TrafficDataLocations};

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

	let traffic_measurements_to_insert = traffic_data.measuring_points
		.iter()
		.map(|point| {
			TrafficMeasurement {
				location_id: point.unique_id,
				observation_time: point.observation_time.into(),
				occupancy_rate: point.calculated_data.occupancy_rate,
				availability_rate: point.calculated_data.availability_rate,
				instability: point.calculated_data.instability
			}
		})
		.collect::<Vec<TrafficMeasurement>>();
	dbg!(&traffic_measurements_to_insert.len());
	TrafficMeasurement::batch_insert(&pool, traffic_measurements_to_insert)
		.await?;

	let traffic_vehicle_measurements_to_insert = traffic_data.measuring_points
		.into_iter()
		.map(|point| {
			point.measurement_data
				.into_iter()
				.map(|vehicle_data| {
					TrafficVehicleMeasurement {
						observation_time: point.observation_time.into(),
						location_id: point.unique_id,
						vehicle_class: vehicle_data.vehicle_class,
						traffic_intensity: vehicle_data.traffic_intensity,
						vehicle_speed_arithmetic: vehicle_data.vehicle_speed_arithmetic,
						vehicle_speed_harmonic: vehicle_data.vehicle_speed_harmonic,
					}
				})
				.collect::<Vec<TrafficVehicleMeasurement>>()
		})
		.flatten()
		.collect::<Vec<TrafficVehicleMeasurement>>();
	dbg!(&traffic_vehicle_measurements_to_insert.len());
	TrafficVehicleMeasurement::batch_insert(&pool, traffic_vehicle_measurements_to_insert)
		.await?;

	Ok(())
}
