use crate::db::models::commit::Commit;
use crate::db::models::file_feature::FileFeature;
use crate::{Connection, Db};
use actix_web::{web, Responder, Result};

pub async fn get_features_between_commits(
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
    let features = FileFeature::fetch_between(from_commit, to_commit, &conn)?;
    Ok(web::Json(features))
}
