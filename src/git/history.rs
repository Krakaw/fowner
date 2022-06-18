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

impl From<&GitHistory> for Commit {
    fn from(history: &GitHistory) -> Self {
        Commit {
            id: 0,
            file_id: 0,
            sha: "".to_string(),
            description: "".to_string(),
            commit_time: (),
            created_at: (),
            updated_at: (),
        }
    }
}
