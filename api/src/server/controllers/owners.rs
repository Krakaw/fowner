use actix_web::{web, Responder, Result};

use crate::db::models::owner::{Owner, UpdateOwner};
use crate::{Connection, Db};

pub async fn get_owners_by_handle(
    db: web::Data<Db>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let db = db.get_ref();
    let conn = Connection::try_from(db)?;
    let owners = Owner::search_by_handle(path.into_inner(), &conn)?;
    Ok(web::Json(owners))
}

pub async fn all(db: web::Data<Db>) -> Result<impl Responder> {
    let db = db.get_ref();
    let conn = Connection::try_from(db)?;
    let owners = Owner::all(&conn)?;
    Ok(web::Json(owners))
}

pub async fn load(db: web::Data<Db>, path: web::Path<u32>) -> Result<impl Responder> {
    let db = db.get_ref();
    let conn = Connection::try_from(db)?;
    let owners = Owner::load(path.into_inner(), &conn)?;
    Ok(web::Json(owners))
}

pub async fn update_owner(
    db: web::Data<Db>,
    path: web::Path<u32>,
    json: web::Json<UpdateOwner>,
) -> Result<impl Responder> {
    let db = db.get_ref();
    let conn = Connection::try_from(db)?;
    let update_owner = json.into_inner();
    let owner = Owner::load(path.into_inner(), &conn)?;
    let owner = owner.update(update_owner, &conn)?;
    Ok(web::Json(owner))
}
