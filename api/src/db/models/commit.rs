use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

use crate::db::models::{extract_all_and_count, extract_first};
use crate::db::Connection;
use crate::errors::FownerError;
use crate::server::paging::SortDir;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Commit {
    pub id: u32,
    pub project_id: u32,
    pub owner_id: u32,
    pub owner_handle: String,
    pub sha: String,
    pub parent_sha: Option<Vec<String>>,
    pub description: String,
    pub commit_time: NaiveDateTime,
    pub feature_names: Vec<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct NewCommit {
    pub owner_id: u32,
    pub project_id: u32,
    pub sha: String,
    pub parent_sha: Option<Vec<String>>,
    pub description: String,
    pub commit_time: NaiveDateTime,
}

impl NewCommit {
    pub fn save(&self, conn: &Connection) -> Result<Commit, FownerError> {
        let mut stmt = conn.prepare(r#"
        INSERT INTO commits (owner_id, project_id, sha, parent_sha, description, commit_time, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%s', 'now'), strftime('%s', 'now'))
        ON CONFLICT
            DO UPDATE SET owner_id    = EXCLUDED.owner_id,
                          parent_sha  = EXCLUDED.parent_sha,
                          description = EXCLUDED.parent_sha,
                          commit_time = EXCLUDED.commit_time,
                          updated_at  = strftime('%s', 'now');
        "#)?;
        let _res = stmt.execute(params![
            self.owner_id,
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
        format!(
            r#"
            SELECT c.id,
                   c.owner_id,
                   coalesce(po.handle, o.handle) AS owner_handle,
                   c.project_id,
                   c.sha,
                   c.parent_sha,
                   c.description,
                   c.commit_time,
                   (SELECT GROUP_CONCAT(f2.name, ',')
                    FROM file_commits fc
                        INNER JOIN file_features ff ON fc.file_id = ff.file_id
                        INNER JOIN features f2 on ff.feature_id = f2.id
                    WHERE fc.commit_id = c.id
                    GROUP BY fc.file_id)                           AS feature_names,

                   c.created_at,
                   c.updated_at,
                   -- This must always be the last column
                   COUNT(*) OVER () AS total_count
            FROM commits c
            LEFT JOIN owners o ON c.owner_id = o.id
            LEFT JOIN owners po ON po.id = o.primary_owner_id
            -- WHERE
            {}
            -- ORDER BY
            {}
            -- PAGING
            {}"#,
            where_clause,
            order_clause.unwrap_or_default(),
            paging_clause.unwrap_or_default()
        )
    }
    pub fn load_by_sha(sha: String, conn: &Connection) -> Result<Self, FownerError> {
        let mut stmt = conn.prepare(&Commit::sql(
            "WHERE c.sha LIKE ?1 ORDER BY c.commit_time ASC;".to_string(),
            None,
            None,
        ))?;
        extract_first!(params![&format!("{}%", sha)], stmt)
    }

    pub fn load(id: i64, conn: &Connection) -> Result<Self, FownerError> {
        let mut stmt = conn.prepare(&Commit::sql("WHERE c.id = ?1;".to_string(), None, None))?;
        extract_first!(params![id], stmt)
    }
    pub fn fetch_latest_for_project(
        project_id: u32,
        conn: &Connection,
    ) -> Result<Self, FownerError> {
        let mut stmt = conn.prepare(&Commit::sql(
            "WHERE c.project_id = ?1".to_string(),
            Some("ORDER BY c.commit_time DESC LIMIT 1".to_string()),
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
    ) -> Result<(i64, Vec<Self>), FownerError> {
        let sort_field = Self::sort_by_field(sort);
        let mut stmt = conn.prepare(&Commit::sql(
            "WHERE c.project_id = ?1 AND (?2 IS NULL OR c.sha LIKE ?2)".to_string(),
            Some(format!(
                "ORDER BY {} {}",
                sort_field,
                sort_dir.unwrap_or_default()
            )),
            Some("LIMIT ?3 OFFSET ?4".to_string()),
        ))?;

        let query = query.map(|query| format!("%{}%", query));
        extract_all_and_count!(params![project_id, query, limit, offset], stmt)
    }

    // pub fn commits_per_handle(
    //     project_id: u32,
    //     conn: &Connection,
    // ) -> Result<HashMap<String, i64>, FownerError> {
    //     let sql = r#"
    //         SELECT
    //             COALESCE (po.handle, o.handle),
    //             COUNT(owner_id) AS commit_count
    //         FROM commits c
    //             JOIN owners o ON o.id = c.owner_id
    //             LEFT JOIN owners po ON po.id = o.primary_owner_id
    //         WHERE c.project_id = ?1
    //         GROUP BY COALESCE (po.handle, o.handle);
    //     "#;
    //     let mut stmt = conn.prepare(sql)?;
    //     let mut rows = stmt.query(params![project_id])?;
    //     let mut result = HashMap::new();
    //     while let Some(row) = rows.next()? {
    //         result.insert(row.get_unwrap(0), row.get_unwrap(1));
    //     }
    //     Ok(result)
    // }
}

impl<'stmt> From<&Row<'stmt>> for Commit {
    fn from(row: &Row) -> Self {
        let feature_names: Vec<String> = row
            .get(8)
            .map(|s: String| s.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_default();
        Self {
            id: row.get(0).unwrap(),
            owner_id: row.get(1).unwrap(),
            owner_handle: row.get(2).unwrap(),
            project_id: row.get(3).unwrap(),
            sha: row.get(4).unwrap(),
            parent_sha: row
                .get(5)
                .map(|s: Option<String>| s.map(|s| s.split(',').map(String::from).collect()))
                .unwrap_or_default(),
            description: row.get(6).unwrap(),
            commit_time: NaiveDateTime::from_timestamp(row.get(7).unwrap(), 0),
            feature_names,
            created_at: NaiveDateTime::from_timestamp(row.get(9).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(10).unwrap(), 0),
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::{Duration, Utc};

    use crate::db::models::commit::{Commit, NewCommit};
    use crate::db::models::file_commit::FileCommit;
    use crate::db::models::owner::NewOwner;
    use crate::server::paging::SortDir;
    use crate::test::builders::file_builder::FileBuilder;
    use crate::test::builders::project_builder::ProjectBuilder;
    use crate::test::tests::TestHandler;
    use crate::Connection;

    #[test]
    fn stats() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = Connection::try_from(db).unwrap();
        let tmp_dir = &handler.tmp_dir;
        let project = ProjectBuilder::with_path(tmp_dir).build(&conn).unwrap();
        let owner = NewOwner {
            handle: "Krakaw".to_string(),
            name: None,
            primary_owner_id: None,
        }
        .save(&conn)
        .unwrap();
        let c1_commit_time = Utc::now().naive_utc();
        NewCommit {
            owner_id: owner.id,
            project_id: project.id,
            sha: "deadbeef".to_string(),
            parent_sha: None,
            description: "Initial Commit".to_string(),
            commit_time: c1_commit_time,
        }
        .save(&conn)
        .unwrap();
        NewCommit {
            owner_id: owner.id,
            project_id: project.id,
            sha: "deadbeef2".to_string(),
            parent_sha: None,
            description: "Another commit".to_string(),
            commit_time: c1_commit_time,
        }
        .save(&conn)
        .unwrap();
        let stats = Commit::commits_per_handle(project.id, &conn).unwrap();
        assert_eq!(stats.get("Krakaw").unwrap(), &2);
    }

    #[test]
    fn save() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = Connection::try_from(db).unwrap();
        let tmp_dir = &handler.tmp_dir;
        let project = ProjectBuilder::with_path(tmp_dir).build(&conn).unwrap();
        let owner = NewOwner {
            handle: "Krakaw".to_string(),
            name: None,
            primary_owner_id: None,
        }
        .save(&conn)
        .unwrap();
        let c1_commit_time = Utc::now().naive_utc();
        let commit_1 = NewCommit {
            owner_id: owner.id,
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
            owner_id: owner.id,
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

        // If a commit with the same sha is re-added update the details
        let commit_override = NewCommit {
            owner_id: owner.id,
            project_id: project.id,
            sha: "deadbeef2".to_string(),
            parent_sha: Some(vec!["deadbeefa".to_string()]),
            description: "Feature Commit 2".to_string(),
            commit_time: Utc::now().naive_utc(),
        }
        .save(&conn)
        .unwrap();
        assert_eq!(commit_override.sha, "deadbeef2".to_string());
        assert_eq!(
            commit_override.parent_sha,
            Some(vec!["deadbeefa".to_string()])
        );
    }

    #[test]
    fn load() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = Connection::try_from(db).unwrap();
        let tmp_dir = &handler.tmp_dir;
        let project = ProjectBuilder::with_path(tmp_dir).build(&conn).unwrap();
        let owner = NewOwner {
            handle: "Krakaw".to_string(),
            name: None,
            primary_owner_id: None,
        }
        .save(&conn)
        .unwrap();
        let owner_id = owner.id;
        let commit_1 = NewCommit {
            owner_id,
            project_id: project.id,
            sha: "deadbeef".to_string(),
            parent_sha: None,
            description: "Initial Commit".to_string(),
            commit_time: Utc::now().naive_utc(),
        }
        .save(&conn)
        .unwrap();
        let commit_2 = NewCommit {
            owner_id,
            project_id: project.id,
            sha: "deadbeef2".to_string(),
            parent_sha: Some(vec!["deadbeef".to_string()]),
            description: "Feature Commit".to_string(),
            commit_time: Utc::now().naive_utc(),
        }
        .save(&conn)
        .unwrap();
        let commit_3 = NewCommit {
            owner_id,
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
        let owner = NewOwner {
            handle: "Krakaw".to_string(),
            name: None,
            primary_owner_id: None,
        }
        .save(&conn)
        .unwrap();
        let owner_id = owner.id;
        let file_1 = FileBuilder {
            project_id: project.id,
            ..FileBuilder::default()
        }
        .with_features(vec!["Feature 1".to_string(), "Feature 2".to_string()])
        .build(&conn)
        .unwrap();

        let mut commit_1 = NewCommit {
            owner_id,
            project_id: project.id,
            sha: "deadbeef".to_string(),
            parent_sha: None,
            description: "Initial Commit".to_string(),
            commit_time: Utc::now().naive_utc(),
        }
        .save(&conn)
        .unwrap();
        FileCommit {
            file_id: file_1.id,
            commit_id: commit_1.id,
        }
        .save(&conn)
        .unwrap();
        commit_1.feature_names = vec!["Feature 1".to_string(), "Feature 2".to_string()];
        let commit_2 = NewCommit {
            owner_id,
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
            owner_id,
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

        let (total, commits_desc) =
            Commit::search(project.id, None, 10, 0, None, None, &conn).unwrap();
        assert_eq!(total, 3);
        assert_eq!(commits_desc.len(), 3);
        assert_eq!(
            commits_desc,
            vec![commit_3.clone(), commit_2.clone(), commit_1.clone()]
        );

        let (total, commits) = Commit::search(
            project.id,
            None,
            10,
            0,
            Some("commit_time".to_string()),
            Some(SortDir::Asc),
            &conn,
        )
        .unwrap();
        assert_eq!(total, 3);
        assert_eq!(commits.len(), 3);
        assert_eq!(
            commits,
            vec![commit_1.clone(), commit_2.clone(), commit_3.clone()]
        );
        let (total, commits) = Commit::search(project.id, None, 2, 0, None, None, &conn).unwrap();
        assert_eq!(total, 3);
        assert_eq!(commits.len(), 2);
        assert_eq!(commits, vec![commit_3.clone(), commit_2.clone()]);
        let (total, commits) =
            Commit::search(project.id, None, 2, 2, None, Some(SortDir::Asc), &conn).unwrap();
        assert_eq!(total, 3);
        assert_eq!(commits.len(), 1);
        assert_eq!(commits, vec![commit_3.clone()]);
        assert_eq!(
            commits.get(0).unwrap().parent_sha,
            Some(vec!["deadbeef2".to_string(), "deadbeef".to_string()])
        );
        let (total, commits) = Commit::search(
            project.id,
            Some("dfe".to_string()),
            50,
            0,
            None,
            None,
            &conn,
        )
        .unwrap();
        assert_eq!(total, 1);
        assert_eq!(commits.len(), 1);
        assert_eq!(commits.first().unwrap().sha, commit_2.sha);
    }
}
