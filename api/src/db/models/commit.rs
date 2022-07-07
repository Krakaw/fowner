use crate::db::models::{extract_all, extract_first};
use crate::db::Connection;
use crate::errors::FownerError;
use crate::server::paging::SortDir;
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Commit {
    pub id: u32,
    pub project_id: u32,
    pub sha: String,
    pub parent_sha: Option<Vec<String>>,
    pub description: String,
    pub commit_time: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct NewCommit {
    pub project_id: u32,
    pub sha: String,
    pub parent_sha: Option<Vec<String>>,
    pub description: String,
    pub commit_time: NaiveDateTime,
}

impl NewCommit {
    pub fn save(&self, conn: &Connection) -> Result<Commit, FownerError> {
        let mut stmt = conn.prepare("INSERT INTO commits (project_id, sha, parent_sha, description, commit_time, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, strftime('%s','now'), strftime('%s','now'))")?;
        let _res = stmt.execute(params![
            self.project_id,
            self.sha,
            self.parent_sha.clone().map(|s| s.join(",")),
            self.description,
            self.commit_time.timestamp()
        ])?;
        let id = conn.last_insert_rowid();
        Commit::load(id, conn)
    }
}
impl Commit {
    fn sort_by_field(field: Option<String>) -> String {
        if let Some(field) = field {
            if vec!["description", "commit_time"].contains(&field.as_str()) {
                return field;
            }
        }

        // Default
        "commit_time".to_string()
    }

    fn sql(
        where_clause: String,
        order_clause: Option<String>,
        paging_clause: Option<String>,
    ) -> String {
        format!("SELECT id, project_id, sha, parent_sha, description, commit_time, created_at, updated_at FROM commits {} {} {}", where_clause, order_clause.unwrap_or_default(), paging_clause.unwrap_or_default())
    }
    pub fn load_by_sha(sha: String, conn: &Connection) -> Result<Self, FownerError> {
        let mut stmt = conn.prepare(&Commit::sql(
            "WHERE sha LIKE ?1 ORDER BY commit_time ASC;".to_string(),
            None,
            None,
        ))?;
        extract_first!(params![&format!("{}%", sha)], stmt)
    }

    pub fn load(id: i64, conn: &Connection) -> Result<Self, FownerError> {
        let mut stmt = conn.prepare(&Commit::sql("WHERE id = ?1;".to_string(), None, None))?;
        extract_first!(params![id], stmt)
    }
    pub fn fetch_latest_for_project(
        project_id: u32,
        conn: &Connection,
    ) -> Result<Self, FownerError> {
        let mut stmt = conn.prepare(&Commit::sql(
            "WHERE project_id = ?1".to_string(),
            Some("ORDER BY commit_time DESC LIMIT 1".to_string()),
            None,
        ))?;
        extract_first!(params![project_id], stmt)
    }
    pub fn search(
        project_id: u32,
        query: Option<String>,
        limit: u32,
        offset: u32,
        sort: Option<String>,
        sort_dir: Option<SortDir>,
        conn: &Connection,
    ) -> Result<Vec<Self>, FownerError> {
        let sort_field = Self::sort_by_field(sort);
        let mut stmt = conn.prepare(&Commit::sql(
            "WHERE project_id = ?1 AND (?2 IS NULL OR sha LIKE ?2)".to_string(),
            Some(format!(
                "ORDER BY {} {}",
                sort_field,
                sort_dir.unwrap_or_default()
            )),
            Some("LIMIT ?3 OFFSET ?4".to_string()),
        ))?;

        let query = query.map(|query| format!("%{}%", query));
        extract_all!(params![project_id, query, limit, offset], stmt)
    }
}

impl<'stmt> From<&Row<'stmt>> for Commit {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            project_id: row.get(1).unwrap(),
            sha: row.get(2).unwrap(),
            parent_sha: row
                .get(3)
                .map(|s: Option<String>| s.map(|s| s.split(',').map(String::from).collect()))
                .unwrap_or_default(),
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
    use crate::server::paging::SortDir;
    use crate::test::builders::project_builder::ProjectBuilder;
    use crate::test::tests::TestHandler;
    use crate::Connection;
    use chrono::{Duration, Utc};

    #[test]
    fn save() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = Connection::try_from(db).unwrap();
        let tmp_dir = &handler.tmp_dir;
        let project = ProjectBuilder::with_path(tmp_dir).build(&conn).unwrap();
        let c1_commit_time = Utc::now().naive_utc();
        let commit_1 = NewCommit {
            project_id: project.id,
            sha: "deadbeef".to_string(),
            parent_sha: None,
            description: "Initial Commit".to_string(),
            commit_time: c1_commit_time,
        }
        .save(&conn)
        .unwrap();
        assert_eq!(commit_1.id, 1);
        assert_eq!(commit_1.project_id, project.id);
        assert_eq!(commit_1.sha, "deadbeef".to_string());
        assert_eq!(commit_1.parent_sha, None);
        assert_eq!(commit_1.description, "Initial Commit".to_string());
        assert_eq!(commit_1.commit_time.timestamp(), c1_commit_time.timestamp());

        let commit_2 = NewCommit {
            project_id: project.id,
            sha: "deadbeef2".to_string(),
            parent_sha: Some(vec!["deadbeef".to_string()]),
            description: "Feature Commit".to_string(),
            commit_time: Utc::now().naive_utc(),
        }
        .save(&conn)
        .unwrap();
        assert_eq!(commit_2.sha, "deadbeef2".to_string());
        assert_eq!(commit_2.parent_sha, Some(vec!["deadbeef".to_string()]));
    }

    #[test]
    fn load() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = Connection::try_from(db).unwrap();
        let tmp_dir = &handler.tmp_dir;
        let project = ProjectBuilder::with_path(tmp_dir).build(&conn).unwrap();
        let commit_1 = NewCommit {
            project_id: project.id,
            sha: "deadbeef".to_string(),
            parent_sha: None,
            description: "Initial Commit".to_string(),
            commit_time: Utc::now().naive_utc(),
        }
        .save(&conn)
        .unwrap();
        let commit_2 = NewCommit {
            project_id: project.id,
            sha: "deadbeef2".to_string(),
            parent_sha: Some(vec!["deadbeef".to_string()]),
            description: "Feature Commit".to_string(),
            commit_time: Utc::now().naive_utc(),
        }
        .save(&conn)
        .unwrap();
        let commit_3 = NewCommit {
            project_id: project.id,
            sha: "deadbeef3".to_string(),
            parent_sha: Some(vec!["deadbeef2".to_string()]),
            description: "Bug Commit".to_string(),
            commit_time: Utc::now()
                .naive_utc()
                .checked_add_signed(Duration::seconds(10))
                .unwrap(),
        }
        .save(&conn)
        .unwrap();

        let c1 = Commit::load(commit_1.id as i64, &conn).unwrap();
        assert_eq!(c1.sha, commit_1.sha);

        let c1 = Commit::load_by_sha("deadbee".to_string(), &conn).unwrap();
        assert_eq!(c1.sha, commit_1.sha);

        let c2 = Commit::load_by_sha("deadbeef2".to_string(), &conn).unwrap();
        assert_eq!(c2.sha, commit_2.sha);
        assert_eq!(c2.parent_sha, Some(vec![commit_1.sha]));

        let c3 = Commit::fetch_latest_for_project(project.id, &conn).unwrap();
        assert_eq!(c3.sha, commit_3.sha);
    }

    #[test]
    fn search() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = Connection::try_from(db).unwrap();
        let tmp_dir = &handler.tmp_dir;
        let project = ProjectBuilder::with_path(tmp_dir).build(&conn).unwrap();
        let commit_1 = NewCommit {
            project_id: project.id,
            sha: "deadbeef".to_string(),
            parent_sha: None,
            description: "Initial Commit".to_string(),
            commit_time: Utc::now().naive_utc(),
        }
        .save(&conn)
        .unwrap();
        let commit_2 = NewCommit {
            project_id: project.id,
            sha: "abcdfe123".to_string(),
            parent_sha: Some(vec!["deadbeef".to_string()]),
            description: "Feature Commit".to_string(),
            commit_time: Utc::now()
                .naive_utc()
                .checked_add_signed(Duration::seconds(5))
                .unwrap(),
        }
        .save(&conn)
        .unwrap();
        let commit_3 = NewCommit {
            project_id: project.id,
            sha: "deadbeef3".to_string(),
            parent_sha: Some(vec!["deadbeef2".to_string(), "deadbeef".to_string()]),
            description: "Bug Commit".to_string(),
            commit_time: Utc::now()
                .naive_utc()
                .checked_add_signed(Duration::seconds(10))
                .unwrap(),
        }
        .save(&conn)
        .unwrap();

        let commits_desc = Commit::search(project.id, None, 10, 0, None, None, &conn).unwrap();
        assert_eq!(commits_desc.len(), 3);
        assert_eq!(
            commits_desc,
            vec![commit_3.clone(), commit_2.clone(), commit_1.clone()]
        );

        let commits = Commit::search(
            project.id,
            None,
            10,
            0,
            Some("commit_time".to_string()),
            Some(SortDir::Asc),
            &conn,
        )
        .unwrap();
        assert_eq!(commits.len(), 3);
        assert_eq!(
            commits,
            vec![commit_1.clone(), commit_2.clone(), commit_3.clone()]
        );
        let commits = Commit::search(project.id, None, 2, 0, None, None, &conn).unwrap();
        assert_eq!(commits.len(), 2);
        assert_eq!(commits, vec![commit_3.clone(), commit_2.clone()]);
        let commits =
            Commit::search(project.id, None, 2, 2, None, Some(SortDir::Asc), &conn).unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits, vec![commit_3.clone()]);
        assert_eq!(
            commits.get(0).unwrap().parent_sha,
            Some(vec!["deadbeef2".to_string(), "deadbeef".to_string()])
        );
        let commits = Commit::search(
            project.id,
            Some("dfe".to_string()),
            50,
            0,
            None,
            None,
            &conn,
        )
        .unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits.first().unwrap().sha, commit_2.sha);
    }
}
