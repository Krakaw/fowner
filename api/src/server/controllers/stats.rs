use actix_web::{web, Responder, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::db::stats::contributions_per_owner::TimeBreakdown;
use crate::{db, Connection, Db};

#[derive(Debug, Serialize, Deserialize)]
pub struct ContributionStatsQuery {
    owner_id: Option<u32>,
    project_id: Option<u32>,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    breakdown: Option<TimeBreakdown>,
    merge_projects: Option<bool>,
}

pub async fn contributions(
    db: web::Data<Db>,
    query: web::Query<ContributionStatsQuery>,
) -> Result<impl Responder> {
    let db = db.get_ref();
    let conn = Connection::try_from(db)?;
    let query = query.into_inner();
    let contributions = db::stats::contributions_per_owner::contributions_per_owner(
        query.owner_id,
        query.project_id,
        query.start,
        query.end,
        query.breakdown,
        query.merge_projects.unwrap_or_default(),
        &conn,
    )?;
    Ok(web::Json(contributions))
}
