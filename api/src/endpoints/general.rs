use crate::structs::AppState;
use crate::utils::{errprint, warnprint, trans_multi, trans_multier, ez};

use actix_web::cookie::{Cookie, time::OffsetDateTime};
use actix_web::{HttpResponse, Error, error, web, post};
use serde::Deserialize;
use sha256;
use sqlx::Acquire;
use tokio::fs::read;

#[derive(Deserialize)]
struct LoginStruct {
    email: String,
    password: String,
}

#[post("/login")]
async fn login(
    state: web::Data<AppState>,
    data: web::Json<LoginStruct>,
    // req: HttpRequest
) -> Result<HttpResponse, Error> {

    // println!("{}", req.headers().get("cookie").unwrap().to_str().unwrap());
    let email = data.email.clone();
    let mut password = data.password.clone();

    let mut er: Option<Error> = None;
    let user: Option<(i32, bool, String)> = match sqlx::query_as("select id, is_active, password from users where email=$1 limit 1;")
        .bind(&email)
        .fetch_one(&state.db)
        .await {
        Ok(a) => {Some(a)},
        Err(sqlx::Error::RowNotFound) => {
            er = Some(error::ErrorBadRequest("Użytkownik nie istnieje."));
            None
        },
        Err(err) => {
            errprint!("{}", err);
            er = Some(error::ErrorInternalServerError("Wystąpił błąd podczas logowania."));
            None
        }
    };

    ez!(er); let user = user.unwrap();

    if user.1 {
        password = format!("{}{}{}", &password[4..7], &password[..], &password[2..4]);
        password = sha256::digest(&password);
    }

    if password != user.2 {
        return Err(error::ErrorBadRequest("Email lub hasło niepoprawne."));    
    }

    let mut response = HttpResponse::Ok().body("Pomyślnie zalogowano");
    response.add_cookie(&Cookie::build("email", email).finish()).unwrap();
    response.add_cookie(&Cookie::build("password", password).finish()).unwrap();

    Ok(response)
}

#[post("/logout")]
async fn logout() -> HttpResponse {

    let mut response = HttpResponse::Ok().body("Pomyślnie wylogowano.");
    response.add_cookie(&Cookie::build("email", "").expires(OffsetDateTime::UNIX_EPOCH).finish()).unwrap();
    response.add_cookie(&Cookie::build("password", "").expires(OffsetDateTime::UNIX_EPOCH).finish()).unwrap();
    response
}

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
            "Błędna nazwa użytkownika lub hasło."
        ));
    }

    let mut conn = state.db.acquire().await.unwrap();
    let mut transaction = conn.begin().await.unwrap();

    let sql = String::from_utf8(read("./sqlv2.sql").await.unwrap()).unwrap();
    let mut is_err = false;
    trans_multi(sql.split(";"), &mut *transaction).await.unwrap_or_else(|err| {
        is_err = true;
        errprint!("SQL error```\n{}```", err)
    });

    if is_err {
        return Err(error::ErrorInternalServerError("Wystąpił błąd podczas resetowania bazy danych."))
    }

    trans_multier!(transaction,
        "INSERT INTO users (
            first_name,
            last_name,
            email,
            password,
            birth_date,
            last_login,
            bio,
            is_active
        ) VALUES (
            'Zbigniew',
            'Kucharski',
            'zbigniew.kucharski@wp.pl',
            '5901d90eeda1aa6d10a930524a24ae350968749e9d33daf07c1c9592fc3a45c0',
            '1969-09-11',
            now(),
            'nie pedał, 100% real',
            true
        );"
        // -- ustaw na dyrektora
        "INSERT INTO users_users_type (
            user_id,
            user_type_id
        ) VALUES (
            1,
            4
        );"
        // -- ustaw na admina
        "INSERT INTO users_users_type (
            user_id,
            user_type_id
        ) VALUES (
            1,
            3
        );"
    );

    transaction.commit().await.unwrap_or_else(|err| {
        errprint!("{}", err);
        is_err = true;
    });

    if is_err {
        return Err(error::ErrorInternalServerError("Wystąpił błąd podczas resetowania bazy danych."))
    }

    warnprint!("Baza danych została zresetowana");
    Ok(HttpResponse::Ok().body("Pomyślnie zresetowano bazę danych."))
}
