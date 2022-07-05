use crate::db::models::commit::Commit;
use crate::server::controllers::SearchRequest;
use crate::Db;
use actix_web::{web, Responder, Result};

pub async fn search(
    db: web::Data<Db>,
    project_id: web::Path<u32>,
    query: web::Query<SearchRequest>,
) -> Result<impl Responder> {
    let query = query.into_inner();
    let project_id = project_id.into_inner();
    let commits = Commit::search(
        project_id,
        query.q,
        query.paging.limit,
        query.paging.offset,
        query.paging.sort,
        query.paging.sort_dir,
        &db,
    )?;

    Ok(web::Json(commits))
}
