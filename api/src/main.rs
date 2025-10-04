use actix_web::{Error, error, HttpRequest, HttpResponse, Responder, web, App, HttpServer, get};
use sqlx::postgres::PgPoolOptions;
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
mod utils;
mod wrappers;
mod appmod;

use self::utils::{get_cookie, get_user};
use self::wrappers::eventor::Eventor;
use self::wrappers::messanger;
use self::structs::{AppState, Env, DbUser};
use self::appmod::*;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // dotenv().ok();
    // let reinit_user = env::var("REINIT_USER").expect("expected .env key: REINIT_USER");
    // let reinit_password = env::var("REINIT_PASSWORD").expect("expected .env key: REINIT_PASSWORD");
    // let dyrek_password = env::var("DYREK_PASSWORD").expect("expected .env key: DYREK_PASSWORD");
    //
    // let pool = PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect("postgres://postgres:postgres@localhost:5432/facecloud")
    //     .await
    //     .expect("Error creating connection pool.");
    //
    // let eventor = Eventor::create(pool.clone());

    let ppa = App::new();
    let app = prod_config(ppa);
    HttpServer::new(move || {
        // App::new()
        //     .app_data(web::Data::new(
        //         AppState {
        //             db: pool.clone(),
        //             sse: Arc::clone(&eventor),
        //             env: Env {
        //                 reinit_user: reinit_user.to_owned(),
        //                 reinit_password: reinit_password.to_owned(),
        //                 dyrek_password: dyrek_password.to_owned()
        //             }
        //         }
        //     ))
        //     .service(index)
        //     // This route is used to listen to events/ sse events
        //     // .route("/events{_:/?}", web::get().to(sse_client))
        //     // This route will create a notification
        //     .service(sse_client)
        //     .service(broadcast_msg)
        //     //# general
        //     .service(dbreinit)
        //     .service(login)
        //     .service(logout)
        //     //# admining users
        //     .service(add_user)
        //     .service(update_user)
        //     .service(delete_user)
        app
    })
    .bind(format!("{}:{}","127.0.0.1", "8000"))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{
        test,
        body::to_bytes,
        web::Bytes
    };

    #[actix_web::test]
    async fn test_index() {
        let app = test::init_service(App::new().service(index)).await;
        let req = test::TestRequest::get()
            .uri("/")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_dbreinit() {

        let app = test::init_service(ppa).await;
        let req = test::TestRequest::post()
            .uri("/dbreinit")
            .set_json(DbreinitStruct {
                user: "dupka".to_string(),
                password: "pupka".to_string()
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(to_bytes(resp.into_body()).await.unwrap(), "Pomyślnie zresetowano bazę danych.");
    }
}
