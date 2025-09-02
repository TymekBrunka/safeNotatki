use actix_web::{Error, HttpRequest, error};
use sqlx::{Acquire, PgConnection, Postgres, pool::PoolConnection};
use std::{fmt::Display, ops::Deref};

macro_rules! errprint {
    () => {
        println!("[\x1b[31mERROR\x1b[0m / \x1b[33m{}\x1b[0m:\x1b[31m{}\x1b[0m]", fil!(), line!())
    };
    ($form:tt) => {{
        println!(
            concat!("[\x1b[31mERROR\x1b[0m / \x1b[33m{}\x1b[0m:\x1b[31m{}\x1b[0m] ", $form), file!(), line!()
        )
    }};
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
    ($form:tt) => {{
        println!(
            concat!("[\x1b[33mWARN \x1b[0m / \x1b[33m{}\x1b[0m:\x1b[31m{}\x1b[0m] ", $form), file!(), line!()
        )
    }};
    ($form:tt, $($arg:tt)*) => {{
        println!(
            concat!("[\x1b[33mWARN \x1b[0m / \x1b[33m{}\x1b[0m:\x1b[31m{}\x1b[0m] ", $form), file!(), line!(), $($arg)*
        )
    }};
}

macro_rules! sucprint {
    () => {
        println!("[\x1b[32mGOOD \x1b[0m / \x1b[33m{}\x1b[0m:\x1b[32m{}\x1b[0m]", fil!(), line!())
    };
    ($form:tt) => {{
        println!(
            concat!("[\x1b[32mGOOD \x1b[0m / \x1b[33m{}\x1b[0m:\x1b[32m{}\x1b[0m] ", $form), file!(), line!()
        )
    }};
    ($form:tt, $($arg:tt)*) => {{
        println!(
            concat!("[\x1b[32mGOOD \x1b[0m / \x1b[33m{}\x1b[0m:\x1b[32m{}\x1b[0m] ", $form), file!(), line!(), $($arg)*
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

macro_rules! ez {
    ($er:tt) => {
        if $er.is_some() {
            return Err($er.unwrap());
        }
    };
}

use crate::structs::DbUser;

pub(crate) use {errprint, warnprint, sucprint, ez, trans_multier};

pub async fn trans_multi(sql: impl IntoIterator<Item: Into<&str>>, transaction: &mut PgConnection) -> Result<(), sqlx::Error> {
    let mut err_string = String::from("");
    let mut er: Option<sqlx::Error> = None;
    for line in sql {
        match sqlx::query(line.into()).execute(&mut *transaction).await {
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

    ez!(er); Ok(())
}

pub async fn get_user(
    db: &mut PoolConnection<Postgres>,
    email: String,
    password: String,
    do_hashing: bool,
) -> Result<DbUser, Error> {
    let mut password = password;

    let conn = db.acquire().await.unwrap();
    let mut er: Option<Error> = None;
    let user: Option<DbUser> = match sqlx::query_as("SELECT * FROM users WHERE email=$1;")
        .bind(email)
        .fetch_optional(&mut *conn)
        .await
    {
        Ok(a) => a,
        Err(sqlx::Error::RowNotFound) => {
            er = Some(error::ErrorBadRequest("Użytkownik nie istnieje."));
            None
        }
        Err(err) => {
            errprint!("{}", err);
            er = Some(error::ErrorInternalServerError(
                "Wystąpił błąd podczas logowania.",
            ));
            None
        }
    };

    ez!(er); let user = user.unwrap();

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
        return Some((email.unwrap(), password.unwrap()));
    }

    None
}

pub trait RemapActix<T, E>
where
    E: Display,
{
    fn remap_actix(self, do_print: bool) -> Result<T, error::Error>;
}

impl<T, E> RemapActix<T, E> for Result<T, E>
where
    E: Display,
{
    fn remap_actix(self, do_print: bool) -> Result<T, error::Error> {
        self.map_err(|err| {
            if do_print {
                errprint!("{}", err);
            }

            error::ErrorInternalServerError("Wystąpił błąd.")
        })
    }
}
