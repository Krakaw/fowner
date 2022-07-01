use crate::db::models::feature::Feature;
use crate::db::models::project::NewProject;
use crate::git::manager::GitManager;
use crate::{Db, File, Processor, Project};
use actix_web::{web, Responder, Result};
use serde_json::json;
use std::path::PathBuf;

pub async fn create(
    db: web::Data<Db>,
    temp_repo_path: web::Data<PathBuf>,
    json: web::Json<NewProject>,
) -> Result<impl Responder> {
    let mut new_project: NewProject = json.into_inner();
    // TODO Don't make this absolute, keep it relative so the folder can be dragged and dropped elsewhere
    if !new_project.path.is_absolute() {
        let project_dir = temp_repo_path.into_inner().join(new_project.path);
        if !project_dir.exists() {
            std::fs::create_dir(project_dir.clone())?;
        }
        new_project.path = project_dir;
    }
    let project = new_project.save(&db)?;

    Ok(web::Json(project))
}

pub async fn fetch_remote_repo(db: web::Data<Db>, path: web::Path<u32>) -> Result<impl Responder> {
    let project_id = path.into_inner();
    let project = Project::load(project_id, &db)?;
    let git_manager = GitManager::init(project.clone().path.into(), project.repo_url)?;
    git_manager.fetch()?;
    let processor = Processor::new(git_manager, &db)?;
    let commit_count = processor.fetch_commits_and_update_db()?;
    Ok(web::Json(json!({ "commits": commit_count })))
}

pub async fn all(db: web::Data<Db>) -> Result<impl Responder> {
    let projects = Project::all(&db)?;
    Ok(web::Json(projects))
}

pub async fn load(db: web::Data<Db>, path: web::Path<u32>) -> Result<impl Responder> {
    let project_id = path.into_inner();
    let project = Project::load(project_id, &db)?;
    let features = Feature::load_by_project(project.id, &db)?;
    let files = File::all(project.id, &db)?;
    Ok(web::Json(
        json!({ "project": project, "features": features, "files": files }),
    ))
}
