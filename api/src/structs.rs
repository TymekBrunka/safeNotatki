use crate::wrappers::eventor::Eventor;

use actix_web_lab::sse::Sender;
use chrono::{NaiveDate, NaiveDateTime};
use sqlx::{Pool, Postgres};
use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

#[derive(Clone)]
pub struct Env {
    pub reinit_user: String,
    pub reinit_password: String,
    pub dyrek_password: String,
}

#[derive(Clone)]
pub struct AppState {
    pub sse: Arc<Eventor>,
    pub db: Pool<Postgres>,
    pub env: Env,
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
    pub id: i32,
    pub index: i32,
    pub sender: Sender,
    pub email: String,
    pub groups: Vec<i32>,
}

pub trait SerDeSer {
    fn ser(self) -> String;
    fn deser(self) -> String;
}

impl SerDeSer for String {
    fn ser(self) -> String {
        self.replace("\\", "\\\\").replace("\"", "\\\"")
    }
    fn deser(self) -> String {
        self.replace("\\\"", "\"").replace("\\\\", "\\")
    }
}

// #[derive(Debug)]
// pub enum NoSseFoundError {
//     NoIdFound(i32),
//     NoEmailFound(String),
// }
//
// #[derive(Debug)]
// pub enum SseError {
//     NoSseFound(NoSseFoundError),
// }
//
// impl Error for NoSseFoundError {}
//
// impl fmt::Display for NoSseFoundError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             Self::NoIdFound(id) => write!(f, "Sender with id {} doesn't exist.", id),
//             Self::NoEmailFound(id) => write!(f, "Sender with email {} doesn't exist.", id),
//         }
//     }
// }
//
// impl Error for SseError {}
//
// impl fmt::Display for SseError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             Self::NoSseFound(err) => write!(f, "{}", err),
//         }
//     }
// }
