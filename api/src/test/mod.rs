pub mod builders;

#[cfg(test)]
pub mod tests {
    use crate::Db;
    use chrono::Utc;
    use r2d2_sqlite::SqliteConnectionManager;
    use std::env::temp_dir;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    pub fn init() -> (Db, PathBuf) {
        let tmp_dir = temp_dir();
        (
            init_test_db(
                tmp_dir
                    .join(format!(
                        "{}.db.sqlite",
                        Utc::now().timestamp_subsec_micros()
                    ))
                    .as_path(),
            ),
            tmp_dir,
        )
    }

    pub fn init_test_db(path: &Path) -> Db {
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
        let sqlite_connection_manager = SqliteConnectionManager::file(path);
        let sqlite_pool = r2d2::Pool::new(sqlite_connection_manager).unwrap();
        let pool = Arc::new(sqlite_pool);
        let db = Db { pool };
        db.init().unwrap();
        db
    }
}
