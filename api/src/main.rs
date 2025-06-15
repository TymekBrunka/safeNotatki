use actix_web::get;
// use actix_web::web::route;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use structs::Env;
// use sqlx::Row;
// use sqlx::Pool;
// use sqlx::Postgres;
use std::sync::Arc;
use actix_web_lab::extract::Path;

use dotenv::dotenv;
use std::env;

mod broadcast;
use self::broadcast::Broadcaster;
mod structs;
use self::structs::AppState;
mod endpoints;
use endpoints::general::*;
pub mod utils;

// SSE
#[get("/events{_:/?}")]
pub async fn sse_client(state: web::Data<AppState>) -> impl Responder {
    state.broadcaster.new_client().await
}

#[get("/events/{msg}")]
pub async fn broadcast_msg(
    state: web::Data<AppState>,
    Path((msg,)): Path<(String,)>,
) -> impl Responder {
    state.broadcaster.broadcast(&msg).await;
    HttpResponse::Ok().body("msg sent")
}


#[get("/")]
async fn index(_req: HttpRequest) -> &'static str {
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    let reinit_user = env::var("REINIT_USER").expect("expected .env key: REINIT_USER");
    let reinit_password = env::var("REINIT_PASSWORD").expect("expected .env key: REINIT_PASSWORD");
    let dyrek_password = env::var("DYREK_PASSWORD").expect("expected .env key: DYREK_PASSWORD");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/facecloud")
        .await
        .expect("Error creating connection pool.");

    let broadcaster = Broadcaster::create();

    // sqlx::query("CREATE DATABASE G;")
    //     .execute(&pool)
    //     .await;
    //

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db: pool.clone(),
                broadcaster: Arc::clone(&broadcaster),
                env: Env{
                    reinit_user: reinit_user.to_owned(),
                    reinit_password: reinit_password.to_owned(),
                    dyrek_password: dyrek_password.to_owned()
                }
            }))
            .service(index)
            // This route is used to listen to events/ sse events
            // .route("/events{_:/?}", web::get().to(sse_client))
            // This route will create a notification
            .service(sse_client)
            .service(broadcast_msg)
            //# general
            .service(dbreinit)
            .service(login)
    })
    .bind(format!("{}:{}","127.0.0.1", "8000"))?
    .run()
    .await
}
