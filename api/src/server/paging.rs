use std::fmt::{Display, Formatter};

use r2d2_sqlite::rusqlite::types::ToSqlOutput;
use r2d2_sqlite::rusqlite::ToSql;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Paging {
    #[serde(default = "missing_limit")]
    #[serde_as(as = "DisplayFromStr")]
    pub limit: u32,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    pub offset: u32,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    pub total: i64,
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub sort_dir: Option<SortDir>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SortDir {
    Asc,
    Desc,
}

impl From<&str> for SortDir {
    fn from(dir: &str) -> Self {
        match dir {
            "Asc" | "asc" => SortDir::Asc,
            "Desc" | "desc" => SortDir::Desc,
            _ => SortDir::Asc,
        }
    }
}

impl ToSql for SortDir {
    fn to_sql(&self) -> r2d2_sqlite::rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            SortDir::Asc => Ok(ToSqlOutput::from("ASC")),
            SortDir::Desc => Ok(ToSqlOutput::from("DESC")),
        }
    }
}

impl Default for SortDir {
    fn default() -> Self {
        Self::Desc
    }
}

impl Display for SortDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SortDir::Asc => "Asc",
            SortDir::Desc => "Desc",
        };
        write!(f, "{}", s)
    }
}

fn missing_limit() -> u32 {
    50
}
