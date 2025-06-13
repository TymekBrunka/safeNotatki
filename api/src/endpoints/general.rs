use crate::structs::AppState;

use actix_web::post;
// use actix_web::HttpRequest;
use actix_web::HttpResponse;
// use actix_web::Responder;
use actix_web::{web, Error, error};
use serde::Deserialize;
use sha256;

#[derive(Deserialize)]
struct DbreinitStruct {
    user: String,
    password: String
}

#[post("/dbreinit")]
async fn dbreinit(state: web::Data<AppState>, data: web::Json<DbreinitStruct>) -> Result<HttpResponse, Error> {
    let password = data.password.clone();
    let password = format!("{}{}{}", &password[4..5], &password[..], &password[2..4]);
    let hash = sha256::digest(&password);
    if !(data.user == state.env.reinit_user 
        && hash == state.env.reinit_password)
    {
        return Err(error::ErrorBadRequest("Błędna nazwa użytkownika lub hasło."));
    }

    sqlx::query_file!("src/sqlv2.sql");

    Ok(HttpResponse::Ok().body("Zresetowano bazę danych."))
}
