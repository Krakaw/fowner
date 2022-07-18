use rusqlite_migration::{Migrations, M};

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(
            r#"
            CREATE TABLE IF NOT EXISTS commits
            (
                id          INTEGER  PRIMARY KEY AUTOINCREMENT,
                owner_id    INTEGER NOT NULL,
                project_id  INTEGER NOT NULL,
                sha         TEXT NOT NULL,
                parent_sha  TEXT NULL,
                description TEXT NULL,
                commit_time INT  NOT NULL,
                created_at  INT  NOT NULL,
                updated_at  INT  NOT NULL,
                CONSTRAINT fk_projects
                    FOREIGN KEY (project_id)
                    REFERENCES projects(id)
                    ON DELETE CASCADE
            );
        "#,
        ),
        M::up(
            r#"
            CREATE UNIQUE INDEX idx_commits_project_id_sha ON commits (project_id, sha);
        "#,
        ),
        M::up(
            r#"
            CREATE TABLE IF NOT EXISTS features
            (
                id          INTEGER  PRIMARY KEY AUTOINCREMENT,
                project_id  INTEGER NOT NULL,
                name        TEXT NOT NULL,
                description TEXT NULL,
                created_at  INT  NOT NULL,
                updated_at  INT  NOT NULL,
                CONSTRAINT fk_projects
                    FOREIGN KEY (project_id)
                    REFERENCES projects(id)
                    ON DELETE CASCADE
            );
        "#,
        ),
        M::up(
            r#"
            CREATE TABLE IF NOT EXISTS files
            (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id  INTEGER NOT NULL,
                path        TEXT    NOT NULL,
                no_features INT DEFAULT 0 NOT NULL,
                created_at  INT     NOT NULL,
                updated_at  INT     NOT NULL,
                CONSTRAINT fk_projects
                    FOREIGN KEY (project_id)
                        REFERENCES projects (id)
                        ON DELETE CASCADE
            );
        "#,
        ),
        M::up(
            r#"
            CREATE UNIQUE INDEX idx_files_project_id_path ON files (project_id, path);
        "#,
        ),
        M::up(
            r#"
            CREATE TABLE IF NOT EXISTS file_owners
            (
                file_id     INTEGER,
                owner_id    INTEGER,
                action_date INT NOT NULL,
                sha         TEXT NOT NULL,
                created_at  INT NOT NULL,
                updated_at  INT NOT NULL,
                CONSTRAINT fk_files
                    FOREIGN KEY (file_id)
                    REFERENCES files(id)
                    ON DELETE CASCADE
            );
        "#,
        ),
        M::up(
            r#"
            CREATE TABLE IF NOT EXISTS file_features
            (
                file_id     INTEGER ,
                feature_id  INTEGER ,
                created_at  INT NOT NULL,
                updated_at  INT NOT NULL,
                CONSTRAINT fk_files
                    FOREIGN KEY (file_id)
                    REFERENCES files(id)
                    ON DELETE CASCADE
            );
        "#,
        ),
        M::up(
            r#"
            CREATE UNIQUE INDEX idx_feature_files_file_id_feature_id ON file_features (file_id, feature_id);
        "#,
        ),
        M::up(
            r#"
            CREATE TABLE IF NOT EXISTS owners
            (
                id               INTEGER PRIMARY KEY AUTOINCREMENT,
                handle           TEXT    NOT NULL UNIQUE,
                name             TEXT    NULL,
                primary_owner_id INTEGER NULL,
                created_at       INT     NOT NULL,
                updated_at       INT     NOT NULL
            );
        "#,
        ),
        M::up(
            r#"
            CREATE TABLE IF NOT EXISTS projects
            (
                id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                path                TEXT NOT NULL UNIQUE,
                name                TEXT NULL,
                repo_url            TEXT NULL,
                github_api_token    TEXT NULL,
                github_labels_only  INT DEFAULT 0 NOT NULL,
                created_at          INT NOT NULL,
                updated_at          INT NOT NULL
            );
        "#,
        ),
        M::up(
            r#"
            CREATE TABLE IF NOT EXISTS file_commits
            (
                file_id     INTEGER,
                commit_id   INTEGER,
                CONSTRAINT fk_files
                    FOREIGN KEY (file_id)
                    REFERENCES files(id)
                    ON DELETE CASCADE

            );
        "#,
        ),
        M::up(
            r#"
            CREATE UNIQUE INDEX idx_file_commits_file_id_commit_id ON file_commits (file_id, commit_id);
        "#,
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_test() {
        assert!(migrations().validate().is_ok());
    }
}
