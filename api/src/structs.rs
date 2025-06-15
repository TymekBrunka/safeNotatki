use sqlx::Pool;
use sqlx::Postgres;
use std::sync::Arc;
use super::broadcast::Broadcaster;

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
