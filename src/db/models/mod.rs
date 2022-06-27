pub mod commit;
pub mod feature;
pub mod file;
pub mod file_commit;
pub mod file_feature;
pub mod file_owner;
pub mod owner;
pub mod project;
pub use crate::errors::FownerError;
macro_rules! extract_first {
    ($params:expr,$stmt:expr) => {
        $stmt
            .query_row($params, |r| Ok(Self::from(r)))
            .map_err(|e| FownerError::from(e))
    };
}

macro_rules! extract_all {
    ($params:expr,$stmt:expr) => {{
        let rows = $stmt.query_map($params, |r| Ok(Self::from(r)))?;
        let mut result = vec![];
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }};
}
pub(crate) use extract_all;
pub(crate) use extract_first;
