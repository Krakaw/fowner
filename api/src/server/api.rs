use crate::server::controllers::{features, owners, projects};
use crate::{Db, FownerError};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use log::info;
use serde_json::json;
use std::net::SocketAddr;
use std::path::PathBuf;

pub struct Api;
const VERSION: &str = env!("CARGO_PKG_VERSION");
impl Api {
    pub async fn start(
        db: Db,
        listen: &SocketAddr,
        storage_path: PathBuf,
    ) -> Result<(), FownerError> {
        info!("Starting server on {:?}", listen);
        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .app_data(web::Data::new(db.clone()))
                .app_data(web::Data::new(storage_path.clone()))
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
                        .route("", web::post().to(projects::create))
                        .service(
                            web::scope("/{project_id}")
                                .route("", web::get().to(projects::load))
                                .service(
                                    web::scope("/fetch")
                                        .route("", web::post().to(projects::fetch_remote_repo)),
                                ),
                        ),
                )
                .service(web::scope("/status").route(
                    "",
                    web::get().to(|| async { web::Json(json!({ "version": VERSION })) }),
                ))
        })
        .bind(listen)?
        .run()
        .await?;
        Ok(())
    }
}
