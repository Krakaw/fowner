use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GitHistory {
    pub handle: String,
    pub hash: String,
    pub timestamp: usize,
    pub summary: String,
    pub files: Vec<String>,
    pub features: Vec<String>,
}
