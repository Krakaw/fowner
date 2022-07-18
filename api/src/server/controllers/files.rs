use actix_web::{web, Responder, Result};

use crate::db::models::commit::Commit;
use crate::db::models::file_commit::FileCommit;
use crate::db::models::file_feature::FileFeature;
use crate::server::controllers::SearchRequest;
use crate::{Connection, Db, File};

pub async fn search(
    db: web::Data<Db>,
    project_id: web::Path<u32>,
    query: web::Query<SearchRequest>,
) -> Result<impl Responder> {
    let db = db.get_ref();
    let conn = Connection::try_from(db)?;
    let query = query.into_inner();
    let project_id = project_id.into_inner();
    let files = File::search(
        project_id,
        query.q.unwrap_or_default(),
        query.paging.limit,
        query.paging.offset,
        &conn,
    )?;

    Ok(web::Json(files))
}

pub async fn remove_features(
    db: web::Data<Db>,
    project_file_id: web::Path<(u32, u32)>,
) -> Result<impl Responder> {
    let db = db.get_ref();
    let conn = Connection::try_from(db)?;
    let (project_id, file_id) = project_file_id.into_inner();
    let file = File::load(project_id, file_id, &conn)?;
    let result = file.remove_features(&conn)?;
    Ok(web::Json(result))
}

pub async fn get_files_between_commits(
    db: web::Data<Db>,
    path: web::Path<(String, String)>,
) -> Result<impl Responder> {
    let db = db.get_ref();
    let conn = Connection::try_from(db)?;
    let (from_commit_str, to_commit_str) = path.into_inner();
    let from_commit = Commit::load_by_sha(from_commit_str, &conn)?;
    let to_commit = Commit::load_by_sha(to_commit_str, &conn)?;
    if from_commit.project_id != to_commit.project_id {
        return Err(actix_web::error::ErrorBadRequest(
            "Commits are from different projects",
        ));
    }
    let (from_commit, to_commit) = if from_commit.commit_time > to_commit.commit_time {
        (to_commit, from_commit)
    } else {
        (from_commit, to_commit)
    };
    let files = FileCommit::fetch_between(from_commit.project_id, from_commit, to_commit, &conn)?;
    Ok(web::Json(files))
}
