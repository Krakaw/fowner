pub mod features;
pub mod owners;

pub struct Server;
use crate::{Db, FownerError};
use actix_web::{web, App, HttpServer};

impl Server {
    pub async fn start(db: Db) -> Result<(), FownerError> {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(db.clone()))
                .service(web::scope("/owners/{owner}").route("", web::get().to(owners::get_owners)))
                .service(web::scope("/features").route(
                    "/{from_commit}/{to_commit}",
                    web::get().to(features::get_features_between_commits),
                ))
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;
        Ok(())
    }
}
