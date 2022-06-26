use crate::{Db, GitRepo, Processor, Project};
use actix_web::{web, Responder, Result};

pub async fn trigger_refresh(db: web::Data<Db>, path: web::Path<u32>) -> Result<impl Responder> {
    let project_id = path.into_inner();
    let project = Project::load(project_id, &db)?;
    let repo = GitRepo {
        path: project.path.clone().into(),
        name: project.name.clone(),
        url: project.repo_url.clone(),
    };
    let mut processor = Processor::new(repo, &db)?;
    let number_of_commits = processor.fetch_commits_and_update_db()?;
    Ok(web::Json(serde_json::json!({
        "numberOfCommits": number_of_commits
    })))
}
