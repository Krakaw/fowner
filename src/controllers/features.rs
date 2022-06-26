use crate::db::models::commit::Commit;
use crate::db::models::file_feature::FileFeature;
use crate::Db;
use actix_web::{web, Responder, Result};

pub async fn get_features_between_commits(
    db: web::Data<Db>,
    path: web::Path<(String, String)>,
) -> Result<impl Responder> {
    let (from_commit_str, to_commit_str) = path.into_inner();
    let from_commit = Commit::load_by_sha(from_commit_str, &db)?;
    let to_commit = Commit::load_by_sha(to_commit_str, &db)?;
    if from_commit.project_id != to_commit.project_id {
        return Err(actix_web::error::ErrorBadRequest(
            "Commits are from different projects",
        ));
    }
    let features = FileFeature::fetch_between(from_commit, to_commit, &db)?;
    Ok(web::Json(features))
}
