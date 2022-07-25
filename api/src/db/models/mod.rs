pub use crate::errors::FownerError;

pub mod commit;
pub mod feature;
pub mod file;
pub mod file_commit;
pub mod file_feature;
pub mod file_owner;
pub mod owner;
pub mod project;

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

macro_rules! extract_all_and_count {
    ($params:expr,$stmt:expr) => {{
        let mut count = 0;
        let col_count = $stmt.column_count();
        let mut rows = $stmt.query($params)?;
        let mut result = vec![];
        while let Some(row) = rows.next()? {
            count = row.get(col_count - 1).unwrap();
            result.push(Self::from(row));
        }
        Ok((count, result))
    }};
}

pub(crate) use extract_all;
pub(crate) use extract_all_and_count;
pub(crate) use extract_first;
