pub mod owners;

pub struct Server;
use crate::Db;
use actix_web::{web, App, HttpServer};

impl Server {
    pub async fn start(db: Db) -> anyhow::Result<()> {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(db.clone()))
                .service(web::scope("/owners/{owner}").route("", web::get().to(owners::get_owners)))
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;
        Ok(())
    }
}
