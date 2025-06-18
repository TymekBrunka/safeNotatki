use crate::wrappers::eventor::Eventor;

use actix_web_lab::sse::Sender;
use chrono::{NaiveDate, NaiveDateTime};
use sqlx::{Pool, Postgres};
use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

pub struct Env {
    pub reinit_user: String,
    pub reinit_password: String,
    pub dyrek_password: String,
}

pub struct AppState {
    pub sse: Arc<Eventor>,
    pub db: Pool<Postgres>,
    pub env: Env,
}

impl AppState {
    pub fn create(pool: Pool<Postgres>, env: Env) -> AppState {
        let eventor: Arc<Eventor> = Eventor::create(RefCell::new(&pool));
        AppState {
            db: pool,
            sse: Arc::clone(&eventor),
            env: env
        }
    }
}

pub enum ErrEnum {
    Http(actix_web::Error),
    Sqlx(sqlx::Error),
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
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct SseUser {
    // pub id: i32,
    pub _id: i32,
    pub sender: Sender,
    pub email: String,
    pub groups: Vec<i32>,
}

#[derive(Debug)]
pub enum SseNotExistError {
    IdNotExists(i32),
    EmailNotExists(String),
}

#[derive(Debug)]
pub enum SseError {
    SseDoesNotExist(SseNotExistError),
}

impl Error for SseNotExistError {}

impl fmt::Display for SseNotExistError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IdNotExists(id) => write!(f, "Socket with id {} doesn't exist.", id),
            Self::EmailNotExists(id) => write!(f, "Socket with email {} doesn't exist.", id),
        }
    }
}

impl Error for SseError {}

impl fmt::Display for SseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SseDoesNotExist(err) => write!(f, "{}", err),
        }
    }
}
