mod features;
mod owners;
mod projects;

pub struct Server;
use crate::{Db, FownerError};
use actix_web::{web, App, HttpServer};

impl Server {
    pub async fn start(db: Db) -> Result<(), FownerError> {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(db.clone()))
                .service(web::scope("/features").route(
                    "/{from_commit}/{to_commit}",
                    web::get().to(features::get_features_between_commits),
                ))
                .service(web::scope("/owners/{owner}").route("", web::get().to(owners::get_owners)))
                .service(
                    web::scope("/projects/{project_id}")
                        .route("", web::put().to(projects::trigger_refresh)),
                )
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;
        Ok(())
    }
}
