use std::num::ParseIntError;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FownerError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Error executing command: {0}")]
    Execution(String),
    #[error("Regex Error: {0}")]
    Regex(#[from] regex::Error),
    #[error("r2d2 pool error: {0}")]
    R2d2(#[from] r2d2::Error),
    #[error("Rusqlite error {0}")]
    Rusqlite(#[from] r2d2_sqlite::rusqlite::Error),
    #[error("File IO Error {0}")]
    FileIO(#[from] std::io::Error),
    #[error("Utf8 Conversion Error: {0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("ParseInt Error: {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("JSON Parse Error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("HTTP Error: {0}")]
    ActixError(#[from] actix_web::Error),
    // #[error("Unknown error")]
    // Unknown,
}

impl actix_web::error::ResponseError for FownerError {}
