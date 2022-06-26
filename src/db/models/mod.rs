pub mod commit;
pub mod feature;
pub mod file;
pub mod file_commit;
pub mod file_feature;
pub mod file_owner;
pub mod owner;
pub mod project;

macro_rules! extract_first {
    ($rows:expr) => {
        if let Some(row) = $rows.next()? {
            Ok(Self::from(row))
        } else {
            Err(FownerError::NotFound("Not found".to_string()))
        }
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
