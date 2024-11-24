use actix_web::{http::StatusCode, HttpResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct AppErrorValue {
	pub(crate) message: String,
	pub(crate) status: u16,
	pub(crate) identifier: String,
	pub(crate) code: String,
}

impl Default for AppErrorValue {
	fn default() -> AppErrorValue {
		AppErrorValue {
			message: "".to_owned(),
			status: 500,
			identifier: "UNIMPLEMENTED".to_owned(),
			code: "ERROR".to_owned(),
		}
	}
}

#[derive(Error, Debug)]
pub enum AppError {
	#[error("Unauthorized: {:#?}", _0)]
	Unauthorized(AppErrorValue),
	#[error("Forbidden: {:#?}", _0)]
	Forbidden(AppErrorValue),
	#[error("Not Found: {:#?}", _0)]
	NotFound(AppErrorValue),
	#[error("Unprocessable Entity: {:#?}", _0)]
	UnprocessableEntity(AppErrorValue),
	#[error("Bad Request: {:#?}", _0)]
	BadRequest(AppErrorValue),
	#[error("Internal Server Error: {:#?}", _0)]
	InternalServerError(AppErrorValue),
}

impl actix_web::error::ResponseError for AppError {
	fn error_response(&self) -> HttpResponse {
		println!("ERROR_RESPONSE: {:?}", self);
		match self {
			AppError::Unauthorized(ref msg) => HttpResponse::Unauthorized().json(msg),
			AppError::Forbidden(ref msg) => HttpResponse::Forbidden().json(msg),
			AppError::NotFound(ref msg) => HttpResponse::NotFound().json(msg),
			AppError::BadRequest(ref msg) => HttpResponse::BadRequest().json(msg),
			AppError::UnprocessableEntity(ref msg) => HttpResponse::UnprocessableEntity().json(msg),
			AppError::InternalServerError(ref msg) => HttpResponse::InternalServerError().json(msg),
		}
	}

	fn status_code(&self) -> StatusCode {
		match *self {
			AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
			AppError::Forbidden(_) => StatusCode::FORBIDDEN,
			AppError::NotFound(_) => StatusCode::NOT_FOUND,
			AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
			AppError::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
			AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}

impl From<std::io::Error> for AppError {
	fn from(err: std::io::Error) -> Self {
		AppError::InternalServerError(AppErrorValue {
			message: err.to_string(),
			status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
			code: "IO_ERROR".to_owned(),
			..Default::default()
		})
	}
}

impl From<sqlx::Error> for AppError {
	fn from(err: sqlx::Error) -> Self {
		AppError::InternalServerError(AppErrorValue {
			message: err.to_string(),
			status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
			code: "SQLX_ERROR".to_owned(),
			..Default::default()
		})
	}
}


impl From<tokio_cron_scheduler::JobSchedulerError> for AppError {
	fn from(err: tokio_cron_scheduler::JobSchedulerError) -> Self {
		AppError::InternalServerError(AppErrorValue {
			message: err.to_string(),
			status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
			code: "SCHEDULER_ERROR".to_owned(),
			..Default::default()
		})
	}
}

impl From<reqwest::Error> for AppError {
	fn from(err: reqwest::Error) -> Self {
		AppError::InternalServerError(AppErrorValue {
			message: err.to_string(),
			status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
			code: "REQWEST_ERROR".to_owned(),
			..Default::default()
		})
	}
}

impl From<quick_xml::de::DeError> for AppError {
	fn from(err: quick_xml::de::DeError) -> Self {
		AppError::InternalServerError(AppErrorValue {
			message: err.to_string(),
			status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
			code: "XML_DE_ERROR".to_owned(),
			..Default::default()
		})
	}
}
