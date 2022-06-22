use crate::db::models::owner::Owner;
use crate::Db;
use actix_web::{web, Responder, Result};

pub async fn get_owners(db: web::Data<Db>, path: web::Path<String>) -> Result<impl Responder> {
    let owners = Owner::search_by_handle(path.into_inner(), &db)?;
    Ok(web::Json(owners))
}
