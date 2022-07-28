use std::net::SocketAddr;
use std::path::PathBuf;

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::dev::{fn_service, ServiceRequest, ServiceResponse};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer};
use log::{info, warn};
use serde_json::json;

use crate::server::controllers::{commits, features, files, owners, projects, stats};
use crate::{Db, FownerError};

pub struct Api;

pub struct AppState {
    public_asset_path: PathBuf,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl Api {
    pub async fn start(
        db: Db,
        listen: &SocketAddr,
        public_asset_path: PathBuf,
        storage_path: PathBuf,
    ) -> Result<(), FownerError> {
        info!("Starting server on {:?}", listen);
        if !public_asset_path.exists() {
            warn!("Public asset path missing, cannot serve frontend.");
        }
        HttpServer::new(move || {
            let app = App::new()
                .wrap(Cors::permissive())
                .wrap(Logger::default())
                .app_data(web::Data::new(db.clone()))
                .app_data(web::Data::new(storage_path.clone()))
                .app_data(web::Data::new(AppState {
                    public_asset_path: public_asset_path.clone(),
                }))
                .service(web::scope("/features").route(
                    "/{from_commit}/{to_commit}",
                    web::get().to(features::get_features_between_commits),
                ))
                .service(web::scope("/files").route(
                    "/{from_commit}/{to_commit}",
                    web::get().to(files::get_files_between_commits),
                ))
                .service(
                    web::scope("/owners")
                        .route("", web::get().to(owners::all))
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
                                .route("", web::delete().to(projects::destroy))
                                .service(
                                    web::scope("/fetch")
                                        .route("", web::post().to(projects::fetch_remote_repo)),
                                )
                                .service(
                                    web::scope("/files")
                                        .route(
                                            "/{file_id}/features",
                                            web::delete().to(files::remove_features),
                                        )
                                        .route("", web::get().to(files::search)),
                                )
                                .service(
                                    web::scope("/commits")
                                        .route("", web::get().to(commits::search)),
                                ),
                        ),
                )
                .service(
                    web::scope("/stats")
                        .route("/contributions", web::get().to(stats::contributions)),
                )
                .service(web::scope("/status").route(
                    "",
                    web::get().to(|| async { web::Json(json!({ "version": VERSION })) }),
                ));

            if !public_asset_path.exists() {
                app
            } else {
                let public_asset_path = public_asset_path.to_string_lossy().to_string();

                app.service(
                    actix_files::Files::new("/", public_asset_path)
                        .index_file("index.html")
                        .default_handler(fn_service(|req: ServiceRequest| async {
                            let (req, _) = req.into_parts();
                            let app_state: Option<&AppState> = req.app_data();
                            let res = if let Some(state) = app_state {
                                let index_path = format!(
                                    "{}/index.html",
                                    state.public_asset_path.to_string_lossy()
                                );
                                let file = NamedFile::open_async(index_path).await?;
                                file.into_response(&req)
                            } else {
                                HttpResponse::NotFound().finish()
                            };
                            Ok(ServiceResponse::new(req, res))
                        })),
                )
            }
        })
        .bind(listen)?
        .run()
        .await?;
        Ok(())
    }
}
