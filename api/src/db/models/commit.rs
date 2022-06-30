use crate::db::models::extract_first;
use crate::errors::FownerError;
use crate::Db;
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};

pub struct Commit {
    pub id: u32,
    pub project_id: u32,
    pub sha: String,
    pub parent_sha: Option<String>,
    pub description: String,
    pub commit_time: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct NewCommit {
    pub project_id: u32,
    pub sha: String,
    pub parent_sha: Option<String>,
    pub description: String,
    pub commit_time: NaiveDateTime,
}

impl NewCommit {
    pub fn save(&self, db: &Db) -> Result<Commit, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO commits (project_id, sha, parent_sha, description, commit_time, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, strftime('%s','now'), strftime('%s','now'))")?;
        let _res = stmt.execute(params![
            self.project_id,
            self.sha,
            self.parent_sha,
            self.description,
            self.commit_time.timestamp()
        ])?;
        let id = conn.last_insert_rowid();
        Commit::load(id, db)
    }
}
impl Commit {
    fn sql(where_clause: String) -> String {
        format!("SELECT id, project_id, sha, parent_sha, description, commit_time, created_at, updated_at FROM commits {}", where_clause)
    }
    pub fn load_by_sha(sha: String, db: &Db) -> Result<Self, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare(&Commit::sql("WHERE sha LIKE ?1;".to_string()))?;
        extract_first!(params![&format!("{}%", sha)], stmt)
    }

    pub fn load(id: i64, db: &Db) -> Result<Self, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare(&Commit::sql("WHERE id = ?1;".to_string()))?;
        extract_first!(params![id], stmt)
    }
    pub fn fetch_latest_for_project(project_id: u32, db: &Db) -> Result<Self, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, project_id, sha, parent_sha, description, commit_time, created_at, updated_at FROM commits WHERE project_id = ?1 ORDER BY commit_time DESC LIMIT 1;")?;
        extract_first!(params![project_id], stmt)
    }
}

impl<'stmt> From<&Row<'stmt>> for Commit {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            project_id: row.get(1).unwrap(),
            sha: row.get(2).unwrap(),
            parent_sha: row.get(3).unwrap(),
            description: row.get(4).unwrap(),
            commit_time: NaiveDateTime::from_timestamp(row.get(5).unwrap(), 0),
            created_at: NaiveDateTime::from_timestamp(row.get(6).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(7).unwrap(), 0),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::db::models::commit::{Commit, NewCommit};
    use crate::test::builders::project_builder::ProjectBuilder;
    use crate::test::tests::init;
    use chrono::{Duration, Utc};

    #[test]
    fn save() {
        let (db, tmp_dir) = init();
        let project = ProjectBuilder::with_path(tmp_dir).build(&db).unwrap();
        let commit_1 = NewCommit {
            project_id: project.id,
            sha: "deadbeef".to_string(),
            parent_sha: None,
            description: "Initial Commit".to_string(),
            commit_time: Utc::now().naive_utc(),
        }
        .save(&db)
        .unwrap();
        assert_eq!(commit_1.sha, "deadbeef".to_string());
        let commit_2 = NewCommit {
            project_id: project.id,
            sha: "deadbeef2".to_string(),
            parent_sha: Some("deadbeef".to_string()),
            description: "Feature Commit".to_string(),
            commit_time: Utc::now().naive_utc(),
        }
        .save(&db)
        .unwrap();
        assert_eq!(commit_2.sha, "deadbeef2".to_string());
        assert_eq!(commit_2.parent_sha, Some("deadbeef".to_string()));
    }

    #[test]
    fn load() {
        let (db, tmp_dir) = init();
        let project = ProjectBuilder::with_path(tmp_dir).build(&db).unwrap();
        let commit_1 = NewCommit {
            project_id: project.id,
            sha: "deadbeef".to_string(),
            parent_sha: None,
            description: "Initial Commit".to_string(),
            commit_time: Utc::now().naive_utc(),
        }
        .save(&db)
        .unwrap();
        let commit_2 = NewCommit {
            project_id: project.id,
            sha: "deadbeef2".to_string(),
            parent_sha: Some("deadbeef".to_string()),
            description: "Feature Commit".to_string(),
            commit_time: Utc::now().naive_utc(),
        }
        .save(&db)
        .unwrap();
        let commit_3 = NewCommit {
            project_id: project.id,
            sha: "deadbeef3".to_string(),
            parent_sha: Some("deadbeef2".to_string()),
            description: "Bug Commit".to_string(),
            commit_time: Utc::now()
                .naive_utc()
                .checked_add_signed(Duration::seconds(10))
                .unwrap(),
        }
        .save(&db)
        .unwrap();

        let c1 = Commit::load(commit_1.id as i64, &db).unwrap();
        assert_eq!(c1.sha, commit_1.sha);

        let c2 = Commit::load_by_sha("deadbeef2".to_string(), &db).unwrap();
        assert_eq!(c2.sha, commit_2.sha);
        assert_eq!(c2.parent_sha, Some(commit_1.sha));

        let c3 = Commit::fetch_latest_for_project(project.id, &db).unwrap();
        assert_eq!(c3.sha, commit_3.sha);
    }
}
