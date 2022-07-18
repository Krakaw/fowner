use std::path::PathBuf;

use chrono::NaiveDateTime;
use log::debug;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

use crate::db::models::feature::NewFeature;
use crate::db::models::file_feature::{FileFeature, NewFileFeature};
use crate::db::models::{extract_all, extract_first};
use crate::db::Connection;
use crate::errors::FownerError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    pub id: u32,
    pub project_id: u32,
    pub path: String,
    pub no_features: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub feature_names: Vec<String>,
    pub commit_shas: Vec<String>,
    pub owners: Vec<String>,
}

impl File {
    pub fn sql(where_clause: Option<String>, limit_clause: Option<String>) -> String {
        format!(
            r#"
            SELECT f.id,
                   f.project_id,
                   f.path,
                   f.no_features,
                   f.created_at,
                   f.updated_at,
                    (SELECT GROUP_CONCAT(f2.name, ',')
                    FROM file_features ff
                             INNER JOIN features f2 on ff.feature_id = f2.id
                    WHERE ff.file_id = f.id
                    GROUP BY ff.file_id)                           AS feature_names,
                   (SELECT GROUP_CONCAT(sha, ',')
                    FROM file_commits fc
                             INNER JOIN commits c on fc.commit_id = c.id
                    WHERE fc.file_id = f.id
                    GROUP BY fc.file_id)                           AS commit_shas,
                   (SELECT GROUP_CONCAT(handle, ',')
                    FROM (SELECT coalesce(po.handle, o.handle) AS handle
                          FROM file_owners fo
                                   INNER JOIN owners o on fo.owner_id = o.id
                                   LEFT JOIN owners po ON po.id = o.primary_owner_id
                          WHERE fo.file_id = f.id
                          GROUP BY coalesce(po.handle, o.handle))) AS owners

            FROM files f
            WHERE f.project_id = ?1
            {}
            GROUP BY f.id
            {};
        "#,
            where_clause.unwrap_or_default(),
            limit_clause.unwrap_or_default()
        )
    }
    pub fn all(project_id: u32, conn: &Connection) -> Result<Vec<File>, FownerError> {
        let mut stmt = conn.prepare(&File::sql(None, None))?;
        extract_all!(params![project_id], stmt)
    }
    pub fn load(project_id: u32, file_id: u32, conn: &Connection) -> Result<File, FownerError> {
        let mut stmt = conn.prepare(&File::sql(Some("AND f.id = ?2".to_string()), None))?;
        extract_first!(params![project_id, file_id], stmt)
    }
    pub fn load_by_path(
        project_id: u32,
        path: String,
        conn: &Connection,
    ) -> Result<File, FownerError> {
        let mut stmt = conn.prepare(&File::sql(Some("AND path = ?2".to_string()), None))?;
        extract_first!(params![project_id, path], stmt)
    }

    pub fn search(
        project_id: u32,
        query: String,
        limit: u32,
        offset: u32,
        conn: &Connection,
    ) -> Result<Vec<File>, FownerError> {
        let mut stmt = conn.prepare(&File::sql(
            Some("AND path LIKE ?2".to_string()),
            Some("LIMIT ?3 OFFSET ?4".to_string()),
        ))?;
        let query = format!("%{}%", query);
        extract_all!(params![project_id, query, limit, offset], stmt)
    }

    pub fn add_feature(
        &self,
        feature_id: u32,
        conn: &Connection,
    ) -> Result<FileFeature, FownerError> {
        if self.no_features {
            return Err(FownerError::FileCannotHaveFeatures(self.path.to_string()));
        }
        NewFileFeature {
            file_id: self.id,
            feature_id,
        }
        .save(conn)
    }

    pub fn remove_features(&self, conn: &Connection) -> Result<usize, FownerError> {
        let sql = "UPDATE files SET no_features = 1 WHERE id = ?1;";
        let mut stmt = conn.prepare(sql)?;
        let result = stmt.execute(params![self.id])?;
        FileFeature::remove_features_from_file(self.id, conn)?;
        Ok(result)
    }

    pub fn generate_feature_file(
        project_id: u32,
        dotfile: PathBuf,
        conn: &Connection,
    ) -> Result<PathBuf, FownerError> {
        // Load any existing file
        let existing_contents = if dotfile.exists() {
            std::fs::read_to_string(dotfile.clone())?
        } else {
            String::new()
        };
        let existing_path_features = existing_contents
            .split('\n')
            .filter_map(|r| {
                let row = r.trim();
                if !row.is_empty() {
                    let parts = row.split('|').collect::<Vec<&str>>();
                    return Some((
                        parts.get(0).cloned().unwrap(),
                        parts
                            .get(1)
                            .cloned()
                            .unwrap()
                            .split(',')
                            .collect::<Vec<&str>>(),
                    ));
                }
                None
            })
            .collect::<Vec<(&str, Vec<&str>)>>();

        let files = Self::all(project_id, conn)?;
        for existing_row in existing_path_features {
            let db_file =
                if let Some(db_file) = files.iter().find(|r| r.path == existing_row.0).cloned() {
                    db_file
                } else {
                    // Create the file
                    NewFile {
                        project_id,
                        path: existing_row.0.to_string(),
                        no_features: false,
                    }
                    .save(conn)?
                };
            for feature_str in existing_row.1 {
                // Check if the features exist, if not create them and attach them to the File
                let feature = NewFeature {
                    project_id,
                    name: feature_str.to_string(),
                    description: None,
                }
                .save(conn)?;
                match db_file.add_feature(feature.id, conn) {
                    Ok(_f) => {}
                    Err(e) => {
                        debug!("{:?}", e);
                    }
                }
            }
        }
        let mut files = Self::all(project_id, conn)?;
        files.sort_by(|a, b| a.path.cmp(&b.path));
        std::fs::write(
            dotfile.clone(),
            files
                .iter()
                .map(|r| format!("{}|{}", r.path.clone(), r.feature_names.join(",")))
                .collect::<Vec<String>>()
                .join("\n"),
        )?;
        Ok(dotfile)
    }
}

pub struct NewFile {
    pub project_id: u32,
    pub path: String,
    pub no_features: bool,
}

impl NewFile {
    pub fn save(&self, conn: &Connection) -> Result<File, FownerError> {
        if let Ok(file) = File::load_by_path(self.project_id, self.path.clone(), conn) {
            return Ok(file);
        };
        let mut stmt = conn.prepare("INSERT INTO files (project_id, path, no_features, created_at, updated_at) VALUES (?1, ?2, ?3, strftime('%s','now'), strftime('%s','now'))")?;
        let _res = stmt.execute(params![
            self.project_id.clone(),
            self.path.clone(),
            self.no_features
        ])?;
        File::load_by_path(self.project_id, self.path.clone(), conn)
    }
}

impl<'stmt> From<&Row<'stmt>> for File {
    fn from(row: &Row) -> Self {
        let feature_names: Vec<String> = row
            .get(6)
            .map(|s: String| s.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let commit_shas: Vec<String> = row
            .get(7)
            .map(|s: String| s.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let owners: Vec<String> = row
            .get(8)
            .map(|s: String| s.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_default();

        Self {
            id: row.get(0).unwrap(),
            project_id: row.get(1).unwrap(),
            path: row.get(2).unwrap(),
            no_features: row.get(3).unwrap(),
            created_at: NaiveDateTime::from_timestamp(row.get(4).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(5).unwrap(), 0),
            feature_names,
            commit_shas,
            owners,
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::Utc;

    use crate::db::models::commit::NewCommit;
    use crate::db::models::feature::NewFeature;
    use crate::db::models::file::NewFile;
    use crate::db::models::file_commit::FileCommit;
    use crate::db::models::file_owner::NewFileOwner;
    use crate::db::models::owner::NewOwner;
    use crate::db::Connection;
    use crate::test::builders::project_builder::ProjectBuilder;
    use crate::test::tests::TestHandler;
    use crate::File;

    #[test]
    fn save() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = &Connection::try_from(db).unwrap();
        let tmp_dir = &handler.tmp_dir;
        let project = ProjectBuilder::with_path(tmp_dir).build(conn).unwrap();
        let file = NewFile {
            project_id: project.id,
            path: "src/main.rs".to_string(),
            no_features: false,
        }
        .save(&conn)
        .unwrap();
        assert_eq!(file.id, 1);
        assert_eq!(file.project_id, project.id);
        assert_eq!(file.path, "src/main.rs".to_string());
        assert!(file.feature_names.is_empty());
        assert!(file.commit_shas.is_empty());
        assert!(file.owners.is_empty());

        // Cannot have duplicate files in the same project
        let file = NewFile {
            project_id: project.id,
            path: "src/main.rs".to_string(),
            no_features: false,
        }
        .save(&conn)
        .unwrap();
        assert_eq!(file.id, 1);
    }

    #[test]
    fn file_without_features() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = &Connection::try_from(db).unwrap();
        let tmp_dir = &handler.tmp_dir;
        let project = ProjectBuilder::with_path(tmp_dir).build(conn).unwrap();
        let file = NewFile {
            project_id: project.id,
            path: "src/main.rs".to_string(),
            no_features: true,
        }
        .save(&conn)
        .unwrap();
        let feature = NewFeature {
            project_id: project.id,
            name: "Test".to_string(),
            description: None,
        }
        .save(&conn)
        .unwrap();

        let file_feature_err = file.add_feature(feature.id, &conn);
        assert!(file_feature_err.is_err());
    }

    #[test]
    fn load() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = &Connection::try_from(db).unwrap();
        let tmp_dir = &handler.tmp_dir;
        let project = ProjectBuilder::with_path(tmp_dir).build(conn).unwrap();
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
        let feature = NewFeature {
            project_id: project.id,
            name: "Test".to_string(),
            description: None,
        }
        .save(&conn)
        .unwrap();
        let owner = NewOwner {
            handle: "Krakaw".to_string(),
            name: None,
            primary_owner_id: None,
        }
        .save(&conn)
        .unwrap();
        let file = NewFile {
            project_id: project.id,
            path: "src/main.rs".to_string(),
            no_features: false,
        }
        .save(&conn)
        .unwrap();

        file.add_feature(feature.id, &conn).unwrap();

        FileCommit {
            file_id: file.id,
            commit_id: commit_1.id,
        }
        .save(&conn)
        .unwrap();
        NewFileOwner {
            file_id: file.id,
            owner_id: owner.id,
            action_date: Utc::now().naive_utc(),
            sha: commit_1.sha,
        }
        .save(&conn)
        .unwrap();

        let db_file = File::load_by_path(project.id, "src/main.rs".to_string(), &conn).unwrap();
        assert_eq!(db_file.id, 1);
        assert_eq!(db_file.project_id, 1);
        assert_eq!(db_file.path, "src/main.rs".to_string());
        assert_eq!(db_file.feature_names, vec!["Test".to_string()]);
        assert_eq!(db_file.commit_shas, vec!["deadbeef".to_string()]);
        assert_eq!(db_file.owners, vec!["Krakaw".to_string()]);

        let commit_2 = NewCommit {
            owner_id,
            project_id: project.id,
            sha: "beefdead".to_string(),
            parent_sha: Some(vec!["deadbeef".to_string()]),
            description: "Feature Commit".to_string(),
            commit_time: Utc::now().naive_utc(),
        }
        .save(&conn)
        .unwrap();

        FileCommit {
            file_id: file.id,
            commit_id: commit_2.id,
        }
        .save(&conn)
        .unwrap();
        let owner = NewOwner {
            handle: "NewOwner".to_string(),
            name: None,
            primary_owner_id: None,
        }
        .save(&conn)
        .unwrap();

        let feature = NewFeature {
            project_id: project.id,
            name: "New Feature".to_string(),
            description: None,
        }
        .save(&conn)
        .unwrap();
        file.add_feature(feature.id, &conn).unwrap();

        NewFileOwner {
            file_id: file.id,
            owner_id: owner.id,
            action_date: Utc::now().naive_utc(),
            sha: commit_2.sha,
        }
        .save(&conn)
        .unwrap();

        let db_file = File::load_by_path(project.id, "src/main.rs".to_string(), &conn).unwrap();
        assert_eq!(
            db_file.feature_names,
            vec!["Test".to_string(), "New Feature".to_string()]
        );
        assert_eq!(
            db_file.commit_shas,
            vec!["deadbeef".to_string(), "beefdead".to_string()]
        );
        assert_eq!(
            db_file.owners,
            vec!["Krakaw".to_string(), "NewOwner".to_string()]
        );
    }
}
