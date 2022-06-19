use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitHistory {
    pub handle: String,
    pub hash: String,
    pub timestamp: usize,
    pub summary: String,
    pub files: Vec<String>,
}
