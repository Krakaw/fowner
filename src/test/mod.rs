#[cfg(test)]
pub mod tests {
    use crate::Db;
    use r2d2_sqlite::SqliteConnectionManager;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use tempfile;
    use tempfile::TempDir;

    pub fn init() -> (Db, TempDir) {
        let tmp_dir = init_test_dir();
        let db_file_path = tempfile::NamedTempFile::new_in(tmp_dir).unwrap();

        (init_test_db(db_file_path.path()), init_test_dir())
    }
    pub fn init_test_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }
    pub fn init_test_db(path: &Path) -> Db {
        let sqlite_connection_manager = SqliteConnectionManager::file(path);
        let sqlite_pool = r2d2::Pool::new(sqlite_connection_manager).unwrap();
        let pool = Arc::new(sqlite_pool);
        let db = Db { pool };
        db.init().unwrap();
        db
    }
}
