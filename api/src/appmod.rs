use actix_web::dev::{ServiceFactory, ServiceRequest};
use dotenv::dotenv;
use std::env;
use sqlx::postgres::PgPoolOptions;
use actix_web::{web, App, Error};
use std::sync::Arc;

use crate::wrappers::eventor::Eventor;
use crate::structs::{AppState, Env};

use crate::endpoints::general::*;
// use crate::endpoints::admining_users::*;

trait MyServiceFactory: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()> {}

impl<T> MyServiceFactory for T
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
    T: actix_service::IntoServiceFactory<dyn ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>, actix_http::Request>,
    T: Sized
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

pub async fn prod_config() -> App<impl MyServiceFactory>
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

    App::new()
    .app_data(web::Data::new(
        AppState {
            db: pool.clone(),
            sse: Arc::clone(&eventor),
            env
        }
    ))
    .service(dbreinit)
}

pub async fn test_config<T>(app_new: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest>,
    // <T as ServiceFactory<ServiceRequest>>::Config = (),
    T: ServiceFactory<ServiceRequest, Config = ()>,
    T: ServiceFactory<ServiceRequest, Error = Error>,
    T: ServiceFactory<ServiceRequest, InitError = ()>,
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

    app_new
    .app_data(web::Data::new(
        AppState {
            db: pool.clone(),
            sse: Arc::clone(&eventor),
            env
        }
    ))
    .service(dbreinit)
}
