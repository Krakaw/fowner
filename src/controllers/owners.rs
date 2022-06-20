use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
#[get("/")]
async fn get_owners() -> impl Responder {
    HttpResponse::Ok().body("")
}
