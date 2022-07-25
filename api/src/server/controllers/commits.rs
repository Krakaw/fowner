use actix_web::{Responder, Result, web};

use crate::{Connection, Db};
use crate::db::models::commit::Commit;
use crate::server::controllers::{PagingResponse, SearchRequest};

pub async fn search(
    db: web::Data<Db>,
    project_id: web::Path<u32>,
    query: web::Query<SearchRequest>,
) -> Result<impl Responder> {
    let query = query.into_inner();
    let project_id = project_id.into_inner();
    let db = db.get_ref();
    let conn = Connection::try_from(db)?;
    let mut paging_response = query.paging.clone();
    let (total_count, commits) = Commit::search(
        project_id,
        query.q,
        query.paging.limit,
        query.paging.offset,
        query.paging.sort,
        query.paging.sort_dir,
        &conn,
    )?;

    paging_response.total = total_count;
    Ok(web::Json(PagingResponse {
        paging: paging_response,
        data: commits,
    }))
}
