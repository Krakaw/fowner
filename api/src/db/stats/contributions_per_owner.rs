use std::collections::HashMap;
use std::str::FromStr;

use chrono::NaiveDate;
use r2d2_sqlite::rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::{Connection, FownerError};

pub type ContributionDateTime = String;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContributionResponse {
    project_id: u32,
    project_name: String,
    start: NaiveDate,
    end: NaiveDate,
    contributions: HashMap<u32, Contributions>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Contributions {
    owner_id: u32,
    owner_handle: String,
    total_contributions: usize,
    contribution_counts: Vec<ContributionCount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContributionCount {
    commit_count: u64,
    commit_time: ContributionDateTime,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum TimeBreakdown {
    #[serde(rename = "daily")]
    Daily,
    #[serde(rename = "monthly")]
    Monthly,
    #[serde(rename = "yearly")]
    Yearly,
}

impl Default for TimeBreakdown {
    fn default() -> Self {
        Self::Daily
    }
}

impl From<String> for TimeBreakdown {
    fn from(t: String) -> Self {
        let t = t.to_lowercase();
        match t.as_str() {
            "daily" => TimeBreakdown::Daily,
            "monthly" => TimeBreakdown::Monthly,
            "yearly" => TimeBreakdown::Yearly,
            _ => TimeBreakdown::Daily,
        }
    }
}

impl From<TimeBreakdown> for String {
    fn from(t: TimeBreakdown) -> Self {
        match t {
            TimeBreakdown::Daily => "%Y-%m-%d",
            TimeBreakdown::Monthly => "%Y-%m",
            TimeBreakdown::Yearly => "%Y",
        }
        .to_string()
    }
}

pub fn contributions_per_owner(
    owner_id: Option<u32>,
    project_id: Option<u32>,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    time_breakdown: Option<TimeBreakdown>,
    conn: &Connection,
) -> Result<HashMap<u32, ContributionResponse>, FownerError> {
    let time_breakdown = time_breakdown.unwrap_or_default();
    let breakdown: String = time_breakdown.into();
    let sql = format!(
        r#"
    SELECT c.project_id,
           p.name                                                   as project_name,
           coalesce(o.primary_owner_id, o.id)                       as owner_id,
           coalesce(po.handle, o.handle)                            as handle,
           strftime('{date_format}', datetime(commit_time, 'unixepoch')) as commit_time_string,
           COUNT(c.id)                                              AS commit_count
    FROM commits c
             JOIN projects p on c.project_id = p.id
             JOIN owners o on c.owner_id = o.id
             LEFT JOIN owners po ON o.primary_owner_id = po.id
    WHERE (?1 IS NULL OR (o.id = ?1 OR o.primary_owner_id = ?1))
    AND (?2 IS NULL OR c.project_id = ?2)
    AND (?3 IS NULL OR commit_time >= ?3)
    AND (?4 IS NULL OR commit_time <= ?4)

    GROUP BY c.project_id, coalesce(o.primary_owner_id, o.id), strftime('{date_format}', datetime(commit_time, 'unixepoch'))
    ORDER BY commit_time;
    "#,
        date_format = breakdown
    );
    let mut stmt = conn.prepare(sql.as_str())?;
    let mut rows = stmt.query(params![
        owner_id,
        project_id,
        start.map(|s| s.and_hms(0, 0, 0).timestamp()),
        end.map(|e| e.and_hms(23, 59, 59).timestamp())
    ])?;
    let mut result = HashMap::new();
    while let Some(row) = rows.next()? {
        let project_id = row.get_unwrap(0);
        let project_name = row.get_unwrap(1);
        let owner_id = row.get_unwrap(2);
        let owner_handle = row.get_unwrap(3);

        let commit_time_string: String = row.get_unwrap(4);

        let date_string = match time_breakdown {
            TimeBreakdown::Daily => format!("{}", &commit_time_string),
            TimeBreakdown::Monthly => format!("{}-01", &commit_time_string),
            TimeBreakdown::Yearly => format!("{}-01-01", &commit_time_string),
        };

        let commit_date = NaiveDate::from_str(&date_string)?;
        let commit_count = row.get_unwrap(5);
        let mut contribution_response = result.entry(project_id).or_insert(ContributionResponse {
            project_id,
            project_name,
            start: commit_date,
            end: commit_date,
            contributions: HashMap::new(),
        });
        if commit_date > contribution_response.end {
            contribution_response.end = commit_date;
        }
        let contribution_count = ContributionCount {
            commit_count,
            commit_time: commit_time_string.to_string(),
        };
        let mut contributions = contribution_response
            .contributions
            .entry(owner_id)
            .or_insert(Contributions {
                owner_id,
                owner_handle,
                total_contributions: 0,
                contribution_counts: vec![],
            });

        contributions.contribution_counts.push(contribution_count);
        contributions.total_contributions += commit_count as usize;
    }
    Ok(result)
}
