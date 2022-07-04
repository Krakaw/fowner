use crate::server::paging::Paging;
use serde::{Deserialize, Serialize};

pub mod commits;
pub mod features;
pub mod files;
pub mod owners;
pub mod projects;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    #[serde(default)]
    q: Option<String>,
    #[serde(flatten)]
    paging: Paging,
}
