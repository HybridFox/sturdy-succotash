
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

	futures::stream::iter(location_data.locations)
		.for_each_concurrent(100, |location| {
		let value = pool.clone();
		async move {
			let location_to_insert = Location {
				latitude: location.latitude,
				longitude: location.longitude,
				location_id: location.unique_id
			};

			Location::insert(&value, location_to_insert)
				.await.unwrap();
		}
		})
		.await;

	futures::stream::iter(traffic_data.measuring_points)
		.for_each_concurrent(100, |point| {
		let value = pool.clone();
		async move {
			futures::stream::iter(point.measurement_data)
				.for_each_concurrent(100, |vehicle_data| {
					let value = value.clone();
					async move {
						let measurement = TrafficVehicleMeasurement {
							observation_time: point.observation_time.into(),
							location_id: point.unique_id,
							vehicle_class: vehicle_data.vehicle_class,
							traffic_intensity: vehicle_data.traffic_intensity,
							vehicle_speed_arithmetic: vehicle_data.vehicle_speed_arithmetic,
							vehicle_speed_harmonic: vehicle_data.vehicle_speed_harmonic,
						};

						TrafficVehicleMeasurement::insert(&value, measurement)
							.await.unwrap();
					}
				})
				.await;

			let measurement = TrafficMeasurement {
				location_id: point.unique_id,
				observation_time: point.observation_time.into(),
				occupancy_rate: point.calculated_data.occupancy_rate,
				availability_rate: point.calculated_data.availability_rate,
				instability: point.calculated_data.instability
			};

			TrafficMeasurement::insert(&value, measurement)
				.await.unwrap()
		}
		})
		.await;

	Ok(())
}
