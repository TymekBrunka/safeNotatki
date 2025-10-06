use actix_web::dev::{ServiceFactory, ServiceRequest};
use dotenv::dotenv;
use sqlx::{Database, Pool, Postgres};
use std::env;
use sqlx::postgres::PgPoolOptions;
use actix_web::{web, App, Error};
use std::sync::Arc;

use crate::wrappers::eventor::{self, Eventor};
use crate::structs::{AppState, Env};

use crate::endpoints::general::*;
// use crate::endpoints::admining_users::*;

// trait MyServiceFactory: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()> {}
//
// impl<T> MyServiceFactory for T
// where
//     T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
//     T: actix_service::IntoServiceFactory<dyn ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>, actix_http::Request>,
//     T: Sized,
//     T: 'static
// {
// }

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
    let dyrek_email = env::var("DYREK_EMAIL").expect("expected .env key: DYREK_EMAIL");
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
        dyrek_email: dyrek_email.to_owned(),
        dyrek_password: dyrek_password.to_owned()
    };
    
    (pool, eventor, env)
}

pub fn init_data(config: (Pool<Postgres>, Arc<Eventor>, Env)) -> web::Data<AppState> {
    web::Data::new(
        AppState {
            db: config.0.clone(),
            sse: Arc::clone(&config.1),
            env: config.2
        }
    )
}
