use crate::structs::{AppState, DbUser};
use crate::utils::{errprint, ez, get_cookie, get_user, sucprint, RemapActix};
use crate::wrappers::user::user_admin;
use crate::wrappers::user::user_admin::get_user_type_ids;

use actix_web_lab::extract::Path;
// use actix_web::cookie::{Cookie, time::OffsetDateTime};
use actix_web::{error, post, web, Error, HttpRequest, HttpResponse};
use serde::Deserialize;
use chrono::NaiveDate;

#[derive(Deserialize)]
struct AddUserStruct {
    first_name: String,
    last_name: String,
    email: String,
    birth_date: NaiveDate,
    user_types: Vec<i32>
}

#[post("/admin/users/add")]
async fn add_user(
    state: web::Data<AppState>,
    data: web::Json<AddUserStruct>,
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    
    let mut conn = state.db.acquire().await.unwrap();

    let cookie = get_cookie(req);
    if cookie.is_none() {
        return Err(error::ErrorUnauthorized("Nie jesteś zalogowany."));
    }
    let cookie = cookie.unwrap();

    let user: DbUser = get_user(&mut conn, cookie.0, cookie.1, true).await?;

    let perms: Vec<i32> = get_user_type_ids(&mut conn, user.id)
        .await
        .remap_actix(true)?;

    if !perms.contains(&3) {
        return Err(error::ErrorUnauthorized("Nie masz uprawnień."))
    }

    if data.user_types.contains(&3) && !perms.contains(&4) {
        return Err(error::ErrorUnauthorized("Operacja niedozwolona."))
    }

    user_admin::add(&mut conn,
        &data.first_name,
        &data.last_name,
        &data.email,
        &data.birth_date,
        &data.user_types
    ).await.remap_actix(true)?;

    sucprint!("zarejestrowano nowego użytkownika");
    Ok(HttpResponse::Ok().body("Pomyślnie zarejestrowano nowego użytkownika."))
}

#[post("/admin/users/delete/{id}")]
async fn delete_user(
    state: web::Data<AppState>,
    Path((id,)): Path<(i32,)>,
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    
    let mut conn = state.db.acquire().await.unwrap();

    let cookie = get_cookie(req);
    if cookie.is_none() {
        return Err(error::ErrorUnauthorized("Nie jesteś zalogowany."));
    }
    let cookie = cookie.unwrap();

    let user: DbUser = get_user(&mut conn, cookie.0, cookie.1, true).await?;

    let perms: Vec<i32> = get_user_type_ids(&mut conn, user.id)
        .await
        .remap_actix(true)?;

    if !perms.contains(&3) {
        return Err(error::ErrorUnauthorized("Nie masz uprawnień."))
    }

    let user_types = user_admin::get_user_type_ids(&mut conn, id).await.remap_actix(true)?;
    if user_types.contains(&3) && !perms.contains(&4) {
        return Err(error::ErrorUnauthorized("Operacja niedozwolona."))
    }

    user_admin::delete(&mut conn, id).await.remap_actix(true)?;

    sucprint!("usunięto użytkownika.");
    Ok(HttpResponse::Ok().body("Pomyślnie usunięto użytkownika."))
}

#[derive(Deserialize)]
struct UpdateUserStruct {
    id: i32,
    first_name: String,
    last_name: String,
    email: String,
    birth_date: NaiveDate,
    user_types: Vec<i32>
}

#[post("/admin/users/update")]
async fn update_user(
    state: web::Data<AppState>,
    data: web::Json<UpdateUserStruct>,
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    
    let mut conn = state.db.acquire().await.unwrap();

    let cookie = get_cookie(req);
    if cookie.is_none() {
        return Err(error::ErrorUnauthorized("Nie jesteś zalogowany."));
    }
    let cookie = cookie.unwrap();

    let user: DbUser = get_user(&mut conn, cookie.0, cookie.1, true).await?;

    let perms: Vec<i32> = get_user_type_ids(&mut conn, user.id)
        .await
        .remap_actix(true)?;

    if !perms.contains(&3) {
        return Err(error::ErrorUnauthorized("Nie masz uprawnień."))
    }

    let user_types = user_admin::get_user_type_ids(&mut conn, data.id).await.remap_actix(true)?;
    if user_types.contains(&3) && !perms.contains(&4) {
        return Err(error::ErrorUnauthorized("Operacja niedozwolona."))
    }

    user_admin::update(&mut conn,
        data.id,
        &data.first_name,
        &data.last_name,
        &data.email,
        data.birth_date,
        &data.user_types
    ).await.remap_actix(true)?;

    sucprint!("zaktualizowano dane użytkownika.");
    Ok(HttpResponse::Ok().body("Pomyślnie zaktualizowano dane użytkownika."))
}
async fn list_users(
    state: web::Data<AppState>,
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    
    let mut conn = state.db.acquire().await.unwrap();

    let cookie = get_cookie(req);
    if cookie.is_none() {
        return Err(error::ErrorUnauthorized("Nie jesteś zalogowany."));
    }
    let cookie = cookie.unwrap();

    let user: DbUser = get_user(&mut conn, cookie.0, cookie.1, true).await?;

    let perms: Vec<i32> = get_user_type_ids(&mut conn, user.id)
        .await
        .remap_actix(true)?;

    if !perms.contains(&3) {
        return Err(error::ErrorUnauthorized("Nie masz uprawnień."))
    }

    let users = user_admin::list(&mut conn).await.remap_actix(true)?;

    Ok(HttpResponse::Ok().body("Pomyślnie zaktualizowano dane użytkownika."))
}

async fn gets_user(
    state: web::Data<AppState>,
    Path((id,)): Path<(i32,)>,
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    
    let mut conn = state.db.acquire().await.unwrap();

    let cookie = get_cookie(req);
    if cookie.is_none() {
        return Err(error::ErrorUnauthorized("Nie jesteś zalogowany."));
    }
    let cookie = cookie.unwrap();

    let user: DbUser = get_user(&mut conn, cookie.0, cookie.1, true).await?;

    let perms: Vec<i32> = get_user_type_ids(&mut conn, user.id)
        .await
        .remap_actix(true)?;

    if !perms.contains(&3) {
        return Err(error::ErrorUnauthorized("Nie masz uprawnień."))
    }

    user_admin::get(&mut conn, id).await.remap_actix(true)?;

    Ok(HttpResponse::Ok().body("Pomyślnie zaktualizowano dane użytkownika."))
}
