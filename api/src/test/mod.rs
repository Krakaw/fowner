pub mod builders;

#[cfg(test)]
pub mod tests {
    use crate::Db;
    use log::debug;
    use r2d2_sqlite::SqliteConnectionManager;
    use rand::Rng;
    use std::env::temp_dir;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    pub struct TestHandler {
        pub db: Db,
        pub tmp_dir: PathBuf,
    }

    impl TestHandler {
        pub fn init() -> Self {
            let tmp_dir = temp_dir().join(rand::thread_rng().gen_range(0..100_000_000).to_string());
            fs::create_dir(&tmp_dir).unwrap();
            let db = TestHandler::init_test_db(tmp_dir.as_path());
            Self { db, tmp_dir }
        }

        fn init_test_db(base_path: &Path) -> Db {
            let path = base_path.join(format!(
                "{}.db.sqlite",
                rand::thread_rng().gen_range(0..100_000_000)
            ));
            let path = path.as_path();
            if path.exists() {
                fs::remove_file(path).unwrap();
            }
            let sqlite_connection_manager = SqliteConnectionManager::file(path);
            let sqlite_pool = r2d2::Pool::new(sqlite_connection_manager).unwrap();
            let pool = Arc::new(sqlite_pool);
            let db = Db { pool };
            db.init().unwrap();
            db
        }
    }

    impl Drop for TestHandler {
        fn drop(&mut self) {
            debug!("Cleaning up test dir: {:?}", &self.tmp_dir);
            fs::remove_dir_all(&self.tmp_dir).unwrap();
        }
    }
}
