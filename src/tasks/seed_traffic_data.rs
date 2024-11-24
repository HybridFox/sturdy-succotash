
use std::env;

use futures::StreamExt;
use quick_xml::de::from_str;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::{errors::AppError, models::traffic_measurements::TrafficMeasurement, TrafficData, TrafficDataLocations};

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

	let measurements = traffic_data.measuring_points
		.iter()
		.map(|point| {
			let location = location_data.locations.iter().find(|location| location.unique_id == point.unique_id);

			match location {
				None => vec![],
				Some(found_location) => point.measurement_data
					.iter()
					.map(|measurement| {
						TrafficMeasurement {
							location_id: point.unique_id,
							latitude: found_location.latitude,
							longitude: found_location.longitude,
							observation_time: point.observation_time.into(),
							vehicle_class: measurement.vehicle_class,
							traffic_intensity: measurement.traffic_intensity,
							vehicle_speed_arithmetic: measurement.vehicle_speed_arithmetic,
							vehicle_speed_harmonic: measurement.vehicle_speed_harmonic,
							occupancy_rate: point.calculated_data.occupancy_rate,
							availability_rate: point.calculated_data.availability_rate,
							instability: point.calculated_data.instability
						}
					})
					.collect::<Vec<TrafficMeasurement>>()
			}
		})
		.flatten()
		.collect::<Vec<TrafficMeasurement>>();

	dbg!(&measurements.len());

	futures::stream::iter(measurements)
		.for_each_concurrent(100, |c| async {
			TrafficMeasurement::insert(&pool, c)
				.await.unwrap()
		})
		.await;

	Ok(())
}
