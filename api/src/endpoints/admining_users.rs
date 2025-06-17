use crate::structs::{AppState, DbUser};
use crate::utils::{errprint, ez, get_cookie, get_user, sucprint, DecupUnwrap, DecupUnwrapActix, UnwrapPerms};
use crate::wrappers::user::user_admin;
use crate::wrappers::user::user_admin::get_user_type_ids;

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
    let mut er: Option<Error> = None;

    let cookie = get_cookie(req);
    if cookie.is_none() {
        return Err(error::ErrorUnauthorized("Nie jesteś zalogowany."));
    }
    let cookie = cookie.unwrap();

    let user: Option<DbUser> = get_user(&mut conn, cookie.0, cookie.1, true).await.decup(&mut er, false);
    ez!(er); let user = user.unwrap();

    let perms: Vec<i32> = get_user_type_ids(&mut conn, user.id)
        .await
        .unwrap_perms(&mut er);
        ez!(er);

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
    ).await.decup_actix(&mut er, true);

    sucprint!("zarejestrowano nowego użytkownika");
    Ok(HttpResponse::Ok().body("Pomyślnie zarejestrowano nowego użytkownika."))
}
