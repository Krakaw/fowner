use crate::db::models::commit::Commit;
use crate::db::models::file::{File, NewFile};
use crate::{Db, GitRepo};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitHistory {
    pub handle: String,
    pub hash: String,
    pub timestamp: usize,
    pub summary: String,
    pub files: Vec<String>,
}

impl GitHistory {
    pub fn store(&self, db: &Db) -> Result<()> {
        // Generate the files
        let files = self.files
        Ok(())
    }
}
