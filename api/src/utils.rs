use serde::de::value::EnumAccessDeserializer;
use sqlx::{pool::PoolConnection, Acquire, PgConnection, Postgres};
use actix_web::{error, Error, HttpRequest};
use std::ops::Deref;

macro_rules! errprint {
    () => {
        println!("[\x1b[31mERROR\x1b[0m / \x1b[33m{}\x1b[0m:\x1b[31m{}\x1b[0m]", fil!(), line!())
    };
    ($form:tt, $($arg:tt)*) => {{
        println!(
            concat!("[\x1b[31mERROR\x1b[0m / \x1b[33m{}\x1b[0m:\x1b[31m{}\x1b[0m] ", $form), file!(), line!(), $($arg)*
        )
    }};
}

macro_rules! warnprint {
    () => {
        println!("[\x1b[33mWARN \x1b[0m / \x1b[33m{}\x1b[0m:\x1b[31m{}\x1b[0m]", fil!(), line!())
    };
    ($form:tt, $($arg:tt)*) => {{
        println!(
            concat!("[\x1b[33mWARN \x1b[0m / \x1b[33m{}\x1b[0m:\x1b[31m{}\x1b[0m] ", $form), file!(), line!(), $($arg)*
        )
    }};
}

macro_rules! trans_multier {
    ($transaction:expr, $($sql:literal)*) => {
        $(
            _ = sqlx::query!($sql).fetch_all(&mut *$transaction).await.unwrap();
        )*
    }
}

use crate::structs::DbUser;

pub(crate) use {errprint, trans_multier};

pub async fn trans_multi(sql: String, transaction: &mut PgConnection) -> Result<(), sqlx::Error> {
    let mut err_string = String::from("");
    let mut er: Option<sqlx::Error> = None;
    for line in sql.split(";") {
        match sqlx::query(line).execute(&mut *transaction).await {
            Ok(_) => {}
            Err(err) => {
                let err_msg = &format!("{}\n", err)[..];
                er = Some(err);
                if err_msg
                    != "error returned from database: current transaction is aborted, commands ignored until end of transaction block\n"
                {
                    err_string += err_msg;
                }
            }
        }
    }

    if er.is_some() {
        return Err(er.unwrap());
    }
    Ok(())
}

pub async fn get_user(
    db: &mut PoolConnection<Postgres>,
    email: String,
    password: String,
    do_hashing: bool
) -> Result<DbUser, Error> {

    let mut password = password;

    let conn = db.acquire().await.unwrap();
    let mut er: Option<Error> = None;
    let user: Option<DbUser> = match sqlx::query_as("SELECT * FROM users WHERE email=$1;")
        .bind(email)
        .fetch_optional(&mut *conn)
        .await {
        Ok(a) => {a},
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

    if er.is_some() {
        return Err(er.unwrap());
    }

    let user = user.unwrap();

    if !do_hashing && user.is_active {
        password = format!("{}{}{}", &password[4..7], &password[..], &password[2..4]);
        password = sha256::digest(&password);
    }

    if password != user.password {
        return Err(error::ErrorBadRequest("Email lub hasło niepoprawne."));    
    }
    
    Ok(user)
}

pub fn get_cookie(req: HttpRequest) -> Option<(String, String)> {
    let mut email: Option<String> = None;
    let mut password: Option<String> = None;
    for cookie in req.cookies().unwrap().deref() {
        if cookie.name() == "email" {
            email = Some(String::from(cookie.value()));
        }

        if cookie.name() == "password" {
            password = Some(String::from(cookie.value()));
        }
    }
    
    if email.is_some() && password.is_some() {
        Some((email.unwrap(), password.unwrap()));
    }

    None
}
