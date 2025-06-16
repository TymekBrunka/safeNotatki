use super::broadcast::Broadcaster;

use sqlx::{Postgres, Pool};
use std::sync::Arc;
use chrono::{DateTime, NaiveDate, Utc};

pub struct Env {
    pub reinit_user: String,
    pub reinit_password: String,
    pub dyrek_password: String
}

pub struct AppState{
    pub broadcaster:Arc<Broadcaster>,
    pub db: Pool<Postgres>,
    pub env: Env
}

pub enum ErrEnum{
    Http(actix_web::Error),
    Sqlx(sqlx::Error)
}

#[derive(sqlx::FromRow, PartialEq, Eq)]
pub struct DbUser {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub birth_date: NaiveDate,
    pub profile_picture: String,
    pub last_login: DateTime<Utc>,
    pub bio: String,
    pub status: bool,
    pub is_active: bool
}

#[derive(sqlx::FromRow, PartialEq, Eq)]
pub struct Single<T> {
    pub value: T
}

trait Unsingler<T> {
    fn unsingle(self) -> Vec<T>;
}

impl<T> Unsingler<T> for Vec<Single<T>> {
    fn unsingle(self) -> Vec<T> {
        self.into_iter().map(|s| s.value).collect()
    }
}

trait ResultUnsingler<T, E> {
    fn unsingle(self) -> Result<Vec<T>, E>;
}

impl<T, E> ResultUnsingler<T, E> for Vec<Single<T>> {
    fn unsingle(self) -> Result<Vec<T>, E> {
        let mut err: Option<E> = None;
        let ret = match self {
            Ok(t) => {t.into_iter().map(|s| s.value).collect()},
            Err(e) => {err = Some(e)}
        }
    }
}
