pub fn migrations() -> Vec<&'static str> {
    vec![
        r#"
            CREATE TABLE IF NOT EXISTS commits
            (
                id          INTEGER  PRIMARY KEY AUTOINCREMENT,
                file_id     INTEGER   NOT NULL,
                sha         TEXT NOT NULL,
                description TEXT NULL,
                commit_time INT  NOT NULL,
                created_at  INT  NOT NULL,
                updated_at  INT  NOT NULL
            );
        "#,
        r#"
            CREATE TABLE IF NOT EXISTS features
            (
                id          INTEGER  PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL,
                description TEXT NULL,
                created_at  INT  NOT NULL,
                updated_at  INT  NOT NULL
            );
        "#,
        r#"
            CREATE TABLE IF NOT EXISTS files
            (
                id         INTEGER  PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                path       TEXT NOT NULL,
                created_at INT  NOT NULL,
                updated_at INT  NOT NULL
            );
        "#,
        r#"
            CREATE UNIQUE INDEX idx_files_project_id_path ON files (project_id, path);
        "#,
        r#"
            CREATE TABLE IF NOT EXISTS file_owners
            (
                file_id     INTEGER,
                owner_id    INTEGER,
                action_date INT NOT NULL,
                created_at  INT NOT NULL,
                updated_at  INT NOT NULL
            );
        "#,
        r#"
            CREATE TABLE IF NOT EXISTS file_features
            (
                file_id     INTEGER ,
                feature_id  INTEGER ,
                created_at  INT NOT NULL,
                updated_at  INT NOT NULL
            );
        "#,
        r#"
            CREATE TABLE IF NOT EXISTS owners
            (
                id            INTEGER  PRIMARY KEY AUTOINCREMENT,
                handle        TEXT NOT NULL UNIQUE,
                name          TEXT NULL,
                created_at    INT  NOT NULL,
                updated_at    INT  NOT NULL
            );
        "#,
        r#"
            CREATE TABLE IF NOT EXISTS projects
            (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                path            TEXT NOT NULL UNIQUE,
                name            TEXT NULL,
                repo_url        TEXT NULL,
                created_at      INT  NOT NULL,
                updated_at      INT  NOT NULL
            );
        "#,
    ]
}
