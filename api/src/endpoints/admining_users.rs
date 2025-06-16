use crate::structs::{AppState, DbUser};
use crate::utils::{get_cookie, get_user}

use actix_web::cookie::{Cookie, time::OffsetDateTime};
use actix_web::{HttpRequest, HttpResponse, Error, error, web, post};
use serde::Deserialize;

#[derive(Deserialize)]
struct AddUserStruct {
    first_name: String,
    last_name: String,
    email: String,
    
}

#[post("/admin/users/add")]
async fn add_user(
    state: web::Data<AppState>,
    data: web::Json<AddUserStruct>,
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    
    if get_cookie(req).is_none() {
        return Err(error::ErrorUnauthorized("Nie jesteś zalogowany."));
    }

    Ok(HttpResponse::Ok().body("Pomyślnie zresetowano bazę danych."))
}
