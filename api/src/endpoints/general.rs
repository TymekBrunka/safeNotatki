use crate::structs::AppState;
use crate::utils::{errprint, trans_multi, trans_multier};

use actix_web::post;
// use actix_web::HttpRequest;
use actix_web::HttpResponse;
// use actix_web::Responder;
use actix_web::{Error, error, web};
use serde::Deserialize;
use sha256;
use sqlx::Acquire;
use tokio::fs::read;

#[derive(Deserialize)]
struct DbreinitStruct {
    user: String,
    password: String,
}

#[post("/dbreinit")]
async fn dbreinit(
    state: web::Data<AppState>,
    data: web::Json<DbreinitStruct>,
) -> Result<HttpResponse, Error> {
    let password = data.password.clone();
    let password = format!("{}{}{}", &password[4..5], &password[..], &password[2..4]);
    let hash = sha256::digest(&password);
    if !(data.user == state.env.reinit_user && hash == state.env.reinit_password) {
        return Err(error::ErrorBadRequest(
            "Błędna nazwa użytkownika lub hasło.",
        ));
    }

    let mut conn = state.db.acquire().await.unwrap();
    let mut transaction = conn.begin().await.unwrap();

    let sql = String::from_utf8(read("./sqlv2.sql").await.unwrap()).unwrap();
    let mut is_err = false;
    match trans_multi(sql, &mut transaction).await{
        Ok(_) => {}
        Err(err) => {
            is_err = true;
            errprint!("SQL error```\n{}```", err)
        }
    };

    _ = sqlx::query!("SELECT first_name from users;");
    transaction.commit().await.unwrap_or_else(|err| {errprint!("{}", err)});
    if is_err { return Err(error::ErrorInternalServerError("Wystąpił błąd podczas resetowania bazy danych."))}
    Ok(HttpResponse::Ok().body("Zresetowano bazę danych."))
}
