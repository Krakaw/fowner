use thiserror::Error;

#[derive(Error, Debug)]
pub enum FownerError {
    #[error("Not found")]
    NotFound,
    #[error("r2d2 pool error")]
    R2d2(#[from] r2d2::Error),
    #[error("Rusqlite error {0}")]
    Rusqlite(#[from] r2d2_sqlite::rusqlite::Error),
    #[error("File IO Error {0}")]
    FileIO(#[from] std::io::Error),
    #[error("Unknown error")]
    Unknown,
}

impl actix_web::error::ResponseError for FownerError {}
