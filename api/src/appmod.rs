use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::{Error, error, HttpRequest, HttpResponse, Responder, web, App, get};
use actix_web_lab::extract::Path;

use dotenv::dotenv;
use sqlx::{Database, Pool, Postgres};
use std::env;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

use crate::utils::{get_cookie, get_user};
use crate::wrappers::eventor::Eventor;
use crate::structs::{AppState, DbUser, Env};

use crate::endpoints::general::*;
use crate::endpoints::admining_users::*;
use crate::wrappers::messanger;

trait MyServiceFactory: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()> {}

impl<T> MyServiceFactory for T
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
    T: actix_service::IntoServiceFactory<dyn ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>, actix_http::Request>,
    T: Sized,
    T: 'static
{
}

// macro_rules! vars_prod {
//     ($pool:expr, $eventor:expr, $env:expr) => {
//         dotenv().ok();
//         let reinit_user = env::var("REINIT_USER").expect("expected .env key: REINIT_USER");
//         let reinit_password = env::var("REINIT_PASSWORD").expect("expected .env key: REINIT_PASSWORD");
//         let dyrek_password = env::var("DYREK_PASSWORD").expect("expected .env key: DYREK_PASSWORD");
//
//         $pool = Some(PgPoolOptions::new()
//             .max_connections(5)
//             .connect("postgres://postgres:postgres@localhost:5432/facecloud")
//             .await
//             .expect("Error creating connection pool."));
//
//         $eventor = Some(Eventor::create($pool.clone().unwrap()));
//
//         $env = Some(Env {
//             reinit_user: reinit_user.to_owned(),
//             reinit_password: reinit_password.to_owned(),
//             dyrek_password: dyrek_password.to_owned()
//         });
//     };
// }
//
// macro_rules! prod_config {
//     ($app_new:expr, $pool:expr, $eventor:expr, $env:expr) => {
//         $app_new
//         .app_data(web::Data::new(
//             AppState {
//                 db: $pool.clone().unwrap(),
//                 sse: Arc::clone($eventor.as_ref().unwrap()),
//                 env: $env.clone().unwrap()
//             }
//         ))
//         .service(dbreinit);
//     };
// }
//
// pub(crate) use {prod_config, vars_prod};

pub async fn prod_config() -> (Pool<Postgres>, Arc<Eventor>, Env)
{
    dotenv().ok();
    let reinit_user = env::var("REINIT_USER").expect("expected .env key: REINIT_USER");
    let reinit_password = env::var("REINIT_PASSWORD").expect("expected .env key: REINIT_PASSWORD");
    let dyrek_password = env::var("DYREK_PASSWORD").expect("expected .env key: DYREK_PASSWORD");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/facecloud")
        .await
        .expect("Error creating connection pool.");

    let eventor = Eventor::create(pool.clone());

    let env = Env {
        reinit_user: reinit_user.to_owned(),
        reinit_password: reinit_password.to_owned(),
        dyrek_password: dyrek_password.to_owned()
    };
    
    (pool, eventor, env)
}

pub fn init_app(config: (Pool<Postgres>, Arc<Eventor>, Env)) -> App<impl MyServiceFactory> {
    App::new()
    .app_data(web::Data::new(
        AppState {
            db: config.0.clone(),
            sse: Arc::clone(&config.1),
            env: config.2
        }
    ))
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
    .service(update_user)
    .service(delete_user)
}

// ----------- endpoints that used to be in main.rs but had to be moved here

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
    let user: DbUser = get_user(&mut conn, cookie.0, cookie.1, true).await?;

    Ok(state.sse.new_client(user.id, user.email).await)
    // Ok(state.sse.new_client(1, "timi".to_string()).await)
}

#[get("/sse/{msg}")]
pub async fn broadcast_msg(
    state: web::Data<AppState>,
    Path((msg,)): Path<(String,)>,
    req: HttpRequest
) -> Result<impl Responder, Error> {

    let cookie = get_cookie(req);
    if cookie.is_none() {
        return Err(error::ErrorUnauthorized("Nie jesteś zalogowany."));
    }

    let cookie = cookie.unwrap();

    let mut conn = state.db.acquire().await.unwrap();
    let user: DbUser = get_user(&mut conn, cookie.0, cookie.1, true).await?;

    let _ = messanger::send(&state.sse, msg, user.id, Some(1), Some(1)).await;
    Ok(HttpResponse::Ok().body("msg sent"))
}


#[get("/")]
async fn index(_req: HttpRequest) -> &'static str {
    "Hello world!"
}
