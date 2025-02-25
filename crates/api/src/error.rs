use actix_web::{
    HttpResponse,
    http::{StatusCode, header},
};
use thiserror::Error;

/// Custom error types for the Nittei API
#[derive(Error, Debug)]
pub enum NitteiError {
    #[error("Internal server error")]
    InternalError,
    #[error("Invalid data provided: Error message: `{0}`")]
    BadClientData(String),
    #[error("There was a conflict with the request. Error message: `{0}`")]
    Conflict(String),
    #[error("Unauthorized request. Error message: `{0}`")]
    Unauthorized(String),
    #[error(
        "Unidentifiable client. Must include the `nittei-account` header. Error message: `{0}`"
    )]
    UnidentifiableClient(String),
    #[error("404 Not found. Error message: `{0}`")]
    NotFound(String),
}

/// Implement the ResponseError trait (from Actix) for the custom error types
/// This allows to automatically convert the error types to HTTP responses
impl actix_web::error::ResponseError for NitteiError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BadClientData(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::UnidentifiableClient(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header((header::CONTENT_TYPE, "text/html; charset=utf-8"))
            .body(self.to_string())
    }
}
