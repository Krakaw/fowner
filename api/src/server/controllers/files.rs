use crate::server::paging::Paging;
use crate::{Db, File};
use actix_web::{web, Responder, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    #[serde(default)]
    q: Option<String>,
    #[serde(flatten)]
    paging: Paging,
}

pub async fn search(
    db: web::Data<Db>,
    project_id: web::Path<u32>,
    query: web::Query<SearchRequest>,
) -> Result<impl Responder> {
    let query = query.into_inner();
    let project_id = project_id.into_inner();
    let files = File::search(
        project_id,
        query.q.unwrap_or_default(),
        query.paging.limit,
        query.paging.offset,
        &db,
    )?;

    Ok(web::Json(files))
}
