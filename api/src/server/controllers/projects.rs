use crate::db::models::project::NewProject;
use crate::db::Connection;
use crate::git::manager::GitManager;
use crate::{Db, FownerError, Processor, Project};
use actix_web::{web, Responder, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FetchRequest {
    pub stop_at_sha: Option<String>,
    pub skip_github_labels: Option<bool>,
}

pub async fn create(db: web::Data<Db>, json: web::Json<NewProject>) -> Result<impl Responder> {
    let mut new_project: NewProject = json.into_inner();
    let db = db.get_ref();
    let conn = Connection::try_from(db)?;
    let repo_url = new_project.repo_url.clone();
    let name = if let Some(name) = new_project.name {
        Some(name)
    } else {
        repo_url.map(|repo_url| {
            repo_url
                .split('/')
                .last()
                .unwrap_or_default()
                .to_string()
                .replace(".git", "")
        })
    };

    new_project.name = name;
    let project = new_project.save(&conn)?;
    debug!("Project created: {}", project.id);
    Ok(web::Json(project))
}

pub async fn fetch_remote_repo(
    db: web::Data<Db>,
    storage_path: web::Data<PathBuf>,
    project_id: web::Path<u32>,
    json: web::Json<FetchRequest>,
) -> Result<impl Responder> {
    let json = json.into_inner();
    let stop_at_sha = json.stop_at_sha;
    let skip_github_labels = json.skip_github_labels.unwrap_or_default();
    debug!(
        "Fetching pulls up until {:?} skipping github labels: {}",
        stop_at_sha, skip_github_labels
    );
    let db = db.get_ref();
    let mut db = db.pool.get().map_err(|e| FownerError::R2d2(e))?;
    let tx = db.transaction().map_err(|e| FownerError::Rusqlite(e))?;
    let conn = Connection::from(tx);
    let project_id = project_id.into_inner();
    let project = Project::load(project_id, &conn)?;
    let absolute_path = project.get_absolute_dir(&storage_path.into_inner(), true)?;
    debug!("Fetching git repo {:?}", absolute_path.to_str());
    let git_manager = GitManager::init(absolute_path, project.repo_url.clone())?;
    git_manager.fetch()?;
    debug!("Fetched git repo");
    let processor = Processor {
        conn: &conn,
        git_manager,
        project,
    };
    debug!("Processing commits");
    let commit_count = processor
        .fetch_commits_and_update_db(stop_at_sha, skip_github_labels)
        .await?;
    conn.transaction()?
        .commit()
        .map_err(|e| FownerError::Rusqlite(e))?;
    debug!("{} commits processed", commit_count);
    Ok(web::Json(json!({ "commits": commit_count })))
}

pub async fn all(db: web::Data<Db>) -> Result<impl Responder> {
    let db = db.get_ref();
    let conn = Connection::try_from(db)?;
    let projects = Project::all(&conn)?;
    Ok(web::Json(projects))
}

pub async fn load(db: web::Data<Db>, path: web::Path<u32>) -> Result<impl Responder> {
    let project_id = path.into_inner();
    let db = db.get_ref();
    let conn = Connection::try_from(db)?;
    let project = Project::load(project_id, &conn)?;
    let display_project = project.for_display(&conn)?;
    Ok(web::Json(json!(display_project)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::project::DisplayProject;
    use crate::test::tests::TestHandler;
    use actix_http::Request;
    use actix_web::dev::Service;
    use actix_web::{dev, error::Error as HttpError, test, web, web::Data, App};
    use serde_json::Value;

    async fn init(
        db: &Db,
        tmp_dir: &PathBuf,
    ) -> impl Service<Request, Response = dev::ServiceResponse, Error = HttpError> {
        test::init_service(
            App::new()
                .app_data(Data::new(db.clone()))
                .app_data(Data::new(tmp_dir.clone()))
                .route("/{id}/fetch", web::post().to(fetch_remote_repo))
                .route("/{id}", web::get().to(load))
                .route("/", web::post().to(create))
                .route("/", web::get().to(all)),
        )
        .await
    }
    #[actix_web::test]
    async fn test_controller() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = Connection::try_from(db).unwrap();
        let app = init(&db, &handler.tmp_dir).await;
        let req = test::TestRequest::post().uri("/").set_json(&json!({"name": "TestProject", "repo_url": "https://github.com/Krakaw/empty.git", "path": "empty", "github_labels_only": false })).to_request();
        let project: Project = test::call_and_read_body_json(&app, req).await;
        assert_eq!(project.id, 1);
        let db_project = Project::load(1, &conn).unwrap();
        assert_eq!(project, db_project.clone());
        let req = test::TestRequest::get().uri("/").to_request();
        let projects: Vec<Project> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(projects.len(), 1);
        assert_eq!(projects, vec![db_project.clone()]);
        let req = test::TestRequest::post()
            .uri("/1/fetch")
            .set_json(json!({}))
            .to_request();
        let commits: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(commits.get("commits").unwrap().as_u64().unwrap(), 1);
        let req = test::TestRequest::get().uri("/1").to_request();
        let project: DisplayProject = test::call_and_read_body_json(&app, req).await;
        assert_eq!(project.project.id, 1);
        assert!(project.features.is_empty());
        assert_eq!(project.files.len(), 1);
        let req = test::TestRequest::post()
            .uri("/1/fetch")
            .set_json(&json!({"stop_at_sha": "no_stop"}))
            .to_request();
        let commits: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(commits.get("commits").unwrap().as_u64().unwrap(), 0);
    }
}
