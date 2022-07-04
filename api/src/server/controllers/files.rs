use crate::server::controllers::SearchRequest;
use crate::{Db, File};
use actix_web::{web, Responder, Result};

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
