use actix_web::get;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::{web, App, HttpServer};
use sqlx::Pool;
use sqlx::Postgres;
mod broadcast;
use self::broadcast::Broadcaster;
use std::{io, sync::Arc};
use actix_web_lab::extract::Path;

use sqlx::postgres::PgPoolOptions;
use sqlx::Row;

pub struct  AppState{
    broadcaster:Arc<Broadcaster>,
    db: Pool<Postgres>
}

// SSE
pub async fn sse_client(state: web::Data<AppState>) -> impl Responder {
    state.broadcaster.new_client().await
}

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
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/safenotatki")
        .await
        .expect("Error creating connection pool.");

    let broadcaster = Broadcaster::create();

    // sqlx::query("CREATE DATABASE G;")
    //     .execute(&pool)
    //     .await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db: pool.clone(),
                broadcaster: Arc::clone(&broadcaster)
            }))
            // This route is used to listen to events/ sse events
            .route("/events{_:/?}", web::get().to(sse_client))
            // This route will create a notification
            .route("/events/{msg}", web::get().to(broadcast_msg))
            .service(index)
    })
    .bind(format!("{}:{}","127.0.0.1", "8000"))?
    .run()
    .await
}
