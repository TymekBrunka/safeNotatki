use sqlx::Pool;
use sqlx::Postgres;
use std::sync::Arc;
use super::broadcast::Broadcaster;

pub struct AppState{
    pub broadcaster:Arc<Broadcaster>,
    pub db: Pool<Postgres>
}
