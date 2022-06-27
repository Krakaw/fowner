use crate::db::models::owner::{Owner, UpdateOwner};
use crate::Db;
use actix_web::{web, Responder, Result};

pub async fn get_owners_by_handle(
    db: web::Data<Db>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let owners = Owner::search_by_handle(path.into_inner(), &db)?;
    Ok(web::Json(owners))
}

pub async fn load(db: web::Data<Db>, path: web::Path<u32>) -> Result<impl Responder> {
    let owners = Owner::load(path.into_inner(), &db)?;
    Ok(web::Json(owners))
}

pub async fn update_owner(
    db: web::Data<Db>,
    path: web::Path<u32>,
    json: web::Json<UpdateOwner>,
) -> Result<impl Responder> {
    let update_owner = json.into_inner();
    let owner = Owner::load(path.into_inner(), &db)?;
    let owner = owner.update(update_owner, &db)?;
    Ok(web::Json(owner))
}
