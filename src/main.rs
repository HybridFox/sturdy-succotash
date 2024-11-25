pub mod errors;
pub mod tasks;
pub mod models;
pub mod state;
pub mod dto;

use std::env;

use actix_web::{get, web, App, HttpResponse, HttpServer, Result};
use chrono::{DateTime, FixedOffset};
use dotenv::dotenv;
use errors::AppError;
use models::traffic_measurement::{FindMeasurementsByLocationIdParams, FindMeasurementsParams, TrafficMeasurement, VehicleClass};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use tasks::seed_traffic_data::seed_traffic_data;
use tokio_cron_scheduler::{Job, JobScheduler};

#[derive(Deserialize)]
pub struct FindAllQueryParams {
	lat: Option<f64>,
	lon: Option<f64>,
	radius: Option<f64>,
	limit: Option<i64>
}

#[get("/measurements")]
pub async fn find_all(
	state: web::Data<AppState>,
	query: web::Query<FindAllQueryParams>,
) -> Result<HttpResponse, AppError> {
	let lat = query.lat;
	let lon = query.lon;
	let radius = query.radius.unwrap_or(1000.0);
	let limit = query.limit.unwrap_or(20);

	let measurements = TrafficMeasurement::get_recent(&state.pool, FindMeasurementsParams {
		lat,
		lon,
		limit,
		radius
	})
		.await?;

	Ok(HttpResponse::Ok().json(measurements))
}

#[derive(Deserialize, Debug)]
pub struct FindByLocationIdPathParams {
	pub location_id: String,
}

#[get("/locations/{location_id}/measurements")]
pub async fn find_by_location_id(
	state: web::Data<AppState>,
	query: web::Query<FindAllQueryParams>,
	params: web::Path<FindByLocationIdPathParams>,
) -> Result<HttpResponse, AppError> {
	let limit = query.limit.unwrap_or(20);

	let measurements = TrafficMeasurement::get_by_location_id(&state.pool, params.location_id.clone(), FindMeasurementsByLocationIdParams {
		limit
	})
		.await?;

	Ok(HttpResponse::Ok().json(measurements))
}


fn deserialize_vehicle_class<'de, D>(deserializer: D) -> Result<VehicleClass, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // First deserialize to string since it's coming from XML attribute
    let class_id_str = String::deserialize(deserializer)?;

    // Parse the string to i32
    let class_id = class_id_str
        .parse::<i32>()
        .map_err(serde::de::Error::custom)?;

    // Use our From implementation to convert to VehicleClass
    Ok(VehicleClass::from(class_id))
}

fn deserialize_dutch_coordinate<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let coordinate = String::deserialize(deserializer)?;
    let coordinate_as_f64 = coordinate
		.replace(",", ".")
        .parse::<f64>()
        .map_err(serde::de::Error::custom)?;
    Ok(coordinate_as_f64)
}

/// Top level structure (MIV = Measuring Instruments for Traffic)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "miv")]
pub struct TrafficData {
    // tijd_publicatie -> publication_time
    #[serde(rename = "tijd_publicatie")]
    pub publication_time: DateTime<FixedOffset>,

    // tijd_laatste_config_wijziging -> last_config_change_time
    #[serde(rename = "tijd_laatste_config_wijziging")]
    pub last_config_change_time: DateTime<FixedOffset>,

    // meetpunt -> measuring_point
    #[serde(rename = "meetpunt")]
    pub measuring_points: Vec<MeasuringPoint>,
}

/// Measuring point structure (Meetpunt)
#[derive(Debug, Serialize, Deserialize)]
pub struct MeasuringPoint {
    // beschrijvende_id -> descriptive_id
    #[serde(rename = "@beschrijvende_id")]
    pub descriptive_id: String,

    // unieke_id -> unique_id
    #[serde(rename = "@unieke_id")]
    pub unique_id: i32,

    // lve_nr -> equipment_number
    #[serde(rename = "lve_nr")]
    pub equipment_number: i32,

    // tijd_waarneming -> observation_time
    #[serde(rename = "tijd_waarneming")]
    pub observation_time: DateTime<FixedOffset>,

    // tijd_laatst_gewijzigd -> last_modified_time
    #[serde(rename = "tijd_laatst_gewijzigd")]
    pub last_modified_time: DateTime<FixedOffset>,

    // actueel_publicatie -> current_publication
    #[serde(rename = "actueel_publicatie")]
    pub current_publication: i32,

    // beschikbaar -> available
    #[serde(rename = "beschikbaar")]
    pub available: i32,

    // defect -> faulty
    #[serde(rename = "defect")]
    pub faulty: i32,

    // geldig -> valid
    #[serde(rename = "geldig")]
    pub valid: i32,

    // meetdata -> measurement_data
    #[serde(rename = "meetdata")]
    pub measurement_data: Vec<MeasurementData>,

    // rekendata -> calculated_data
    #[serde(rename = "rekendata")]
    pub calculated_data: CalculatedData,
}

/// Measurement data structure (Meetdata)
#[derive(Debug, Serialize, Deserialize)]
pub struct MeasurementData {
    #[serde(rename = "@klasse_id", deserialize_with = "deserialize_vehicle_class")]
    pub vehicle_class: VehicleClass,

    // verkeersintensiteit -> traffic_intensity
    #[serde(rename = "verkeersintensiteit")]
    pub traffic_intensity: i32,

    // voertuigsnelheid_rekenkundig -> vehicle_speed_arithmetic
    #[serde(rename = "voertuigsnelheid_rekenkundig")]
    pub vehicle_speed_arithmetic: i32,

    // voertuigsnelheid_harmonisch -> vehicle_speed_harmonic
    #[serde(rename = "voertuigsnelheid_harmonisch")]
    pub vehicle_speed_harmonic: i32,
}

/// Calculated data structure (Rekendata)
#[derive(Debug, Serialize, Deserialize)]
pub struct CalculatedData {
    // bezettingsgraad -> occupancy_rate
    #[serde(rename = "bezettingsgraad")]
    pub occupancy_rate: i32,

    // beschikbaarheidsgraad -> availability_rate
    #[serde(rename = "beschikbaarheidsgraad")]
    pub availability_rate: i32,

    // onrustigheid -> instability
    #[serde(rename = "onrustigheid")]
    pub instability: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "mivconfig")]
pub struct TrafficDataLocations {
    #[serde(rename = "tijd_laatste_config_wijziging")]
    pub publication_time: DateTime<FixedOffset>,
    #[serde(rename = "meetpunt")]
    pub locations: Vec<MeasuringPointLocation>,
}

/// Measuring point structure (Meetpunt)
#[derive(Debug, Serialize, Deserialize)]
pub struct MeasuringPointLocation {
    #[serde(rename = "@unieke_id")]
    pub unique_id: i32,
    #[serde(rename = "breedtegraad_EPSG_4326", deserialize_with = "deserialize_dutch_coordinate")]
	pub latitude: f64,
    #[serde(rename = "lengtegraad_EPSG_4326", deserialize_with = "deserialize_dutch_coordinate")]
    pub longitude: f64,
}

#[actix_web::main]
async fn main() -> std::result::Result<(), AppError> {
    dotenv().ok();

    let scheduler = JobScheduler::new().await?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").expect("DATABASE_URL missing"))
        .await?;

    // Add basic cron job
    scheduler.add(
		Job::new_async("0 * * * * *", |_uuid, _l| {
			println!("yolo");
            Box::pin(async move {
				seed_traffic_data()
					.await
					.unwrap()
			})
        })?
    ).await?;

    scheduler.start().await?;

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool)
        .await?;

    dbg!(&row);
	let state: AppState = {
		let pool = PgPoolOptions::new()
			.max_connections(5)
			.connect(&env::var("DATABASE_URL").expect("DATABASE_URL missing"))
			.await?;

		AppState { pool }
	};

    let _ = HttpServer::new(move || App::new()
		.service(find_all)
		.service(find_by_location_id)
		.app_data(actix_web::web::Data::new(state.clone()))
	)
        .bind(("0.0.0.0", 8080))?
        .run()
        .await;

    Ok(())
}
