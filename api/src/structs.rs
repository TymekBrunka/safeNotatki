use crate::wrappers::eventor::Eventor;

use sqlx::{Postgres, Pool};
use std::sync::Arc;
use chrono::{NaiveDateTime, NaiveDate};
use actix_web_lab::sse::Sender;

pub struct Env {
    pub reinit_user: String,
    pub reinit_password: String,
    pub dyrek_password: String
}

pub struct AppState{
    pub sse:Arc<Eventor>,
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
    pub profile_picture: Option<String>,
    pub last_login: NaiveDateTime,
    pub bio: String,
    pub status: bool,
    pub is_active: bool
}

#[derive(Debug, Clone)]
pub struct SseUser {
    // pub id: i32,
    pub sender: Sender,
    pub email: String,
    pub groups: Vec<i32>
}
