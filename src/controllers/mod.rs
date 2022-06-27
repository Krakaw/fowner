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
                .service(
                    web::scope("/owners")
                        .service(
                            web::scope("/search/{owner_handle}")
                                .route("", web::get().to(owners::get_owners_by_handle)),
                        )
                        .service(
                            web::scope("/{id}")
                                .route("", web::put().to(owners::update_owner))
                                .route("", web::get().to(owners::load)),
                        ),
                )
                .service(
                    web::scope("/projects")
                        .route("", web::get().to(projects::all))
                        .service(
                            web::scope("/{project_id}")
                                .route("", web::put().to(projects::trigger_refresh))
                                .route("", web::get().to(projects::load)),
                        ),
                )
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;
        Ok(())
    }
}
