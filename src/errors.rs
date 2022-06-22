use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FownerError {
    #[error("Not found")]
    NotFound,
    #[error("r2d2 pool error")]
    R2d2(#[from] r2d2::Error),
    #[error("Rusqlite error {0}")]
    Rusqlite(#[from] r2d2_sqlite::rusqlite::Error),

    #[error("Unknown error")]
    Unknown,
}

// impl Display for FownerError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

impl actix_web::error::ResponseError for FownerError {}
