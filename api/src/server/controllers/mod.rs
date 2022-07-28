use serde::{Deserialize, Serialize};

use crate::server::paging::Paging;

pub mod commits;
pub mod features;
pub mod files;
pub mod owners;
pub mod projects;
pub mod stats;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    #[serde(default)]
    q: Option<String>,
    #[serde(flatten)]
    paging: Paging,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PagingResponse<T> {
    paging: Paging,
    data: Vec<T>,
}
