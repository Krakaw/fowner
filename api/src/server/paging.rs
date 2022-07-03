use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Paging {
    #[serde(default = "missing_limit")]
    #[serde_as(as = "DisplayFromStr")]
    pub limit: u32,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    pub offset: u32,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    pub total: u64,
}

fn missing_limit() -> u32 {
    50
}
