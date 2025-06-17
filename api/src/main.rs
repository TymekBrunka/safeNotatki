use actix_web::error;
use actix_web::get;
use actix_web::Error;
// use actix_web::web::route;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use structs::DbUser;
use structs::Env;
use utils::ez;
use utils::get_cookie;
use utils::get_user;
use utils::DecupUnwrap;
// use sqlx::Row;
// use sqlx::Pool;
// use sqlx::Postgres;
use std::sync::Arc;
use actix_web_lab::extract::Path;

use dotenv::dotenv;
use std::env;

// definiowanie rustowi że ma moduły
mod structs;
mod endpoints;
pub mod utils;
pub mod wrappers;

use self::wrappers::eventor::Eventor;
use self::structs::AppState;

use endpoints::general::*;
use endpoints::admining_users::*;

// SSE
#[get("/sse{_:/?}")]
pub async fn sse_client(
    state: web::Data<AppState>,
    req: HttpRequest
) -> Result<impl Responder, Error> {

    let cookie = get_cookie(req);
    if cookie.is_none() {
        return Err(error::ErrorUnauthorized("Nie jesteś zalogowany."));
    }

    let cookie = cookie.unwrap();

    let mut conn = state.db.acquire().await.unwrap();
    let mut er: Option<Error> = None;
    let user: Option<DbUser> = get_user(&mut conn, cookie.0, cookie.1, true).await.decup(&mut er, false);
    ez!(er); let user = user.unwrap();

    Ok(state.sse.new_client(user.id, user.email).await)
    // Ok(state.broadcaster.new_client(1, "timi".to_string()).await)
}

#[get("/see/{msg}")]
pub async fn broadcast_msg(
    state: web::Data<AppState>,
    Path((msg,)): Path<(String,)>,
) -> impl Responder {
    state.sse.broadcast(&msg).await;
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

    let eventor = Eventor::create();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db: pool.clone(),
                sse: Arc::clone(&eventor),
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
            .service(logout)
            //# admining users
            .service(add_user)
    })
    .bind(format!("{}:{}","127.0.0.1", "8000"))?
    .run()
    .await
}
