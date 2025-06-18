use std::{cell::RefCell, collections::HashMap, sync::{Arc, RwLock}, time::Duration};
use actix_web::rt::time::interval;
use actix_web_lab::sse::{self, ChannelStream, Sse};
use futures_util::future;
use sqlx::{pool::PoolConnection, Acquire, Pool, Postgres};
// use parking_lot::Mutex;

use crate::utils::{ez, DecupUnwrap};
use crate::structs::SseUser;
use crate::utils::sucprint;

async fn get_group_ids(
    db: &mut PoolConnection<Postgres>, userid: i32
) -> Result<Vec<i32>, sqlx::Error> {
    let conn = db.acquire().await.unwrap();
    let mut er: Option<sqlx::Error> = None;
    let ids: Option<Vec<i32>> = sqlx::query_scalar("SELECT id FROM groups JOIN group_members ON groups.id = group_members.group_id WHERE group_members.user_id = $1;")
        .bind(userid)
        .fetch_all(&mut *conn)
        .await
        .decup(&mut er, true);

    ez!(er); Ok(ids.unwrap())
}

pub struct Eventor {
    db: RefCell<Pool<Postgres>>,
    inner: RwLock<EventorData>,
}

#[derive(Debug, Clone)]
struct EventorData {
    index: i32,
    clients: HashMap<i32, SseUser>
}

impl Eventor {
    /// Constructs new broadcaster and spawns ping loop.
    pub fn create(db: &Pool<Postgres>) -> Arc<Self> {
        let this = Arc::new(Eventor {
            db: RefCell::new(db),
            inner: RwLock::new(
                EventorData{
                    index: 0,
                    clients: HashMap::new()
                }
            ),
        });
        Eventor::spawn_ping(Arc::clone(&this));
        sucprint!("SSE server initialized");

        this
    }

    /// Pings clients every 10 seconds to see if they are alive and remove them from the broadcast list if not.
    fn spawn_ping(this: Arc<Self>) {
        actix_web::rt::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));

            loop {
                interval.tick().await;
                this.remove_stale_clients().await;
            }
        });
    }

    /// Removes all non-responsive clients from broadcast list.
    async fn remove_stale_clients(&self) {
        
        let clients = self.inner.read().unwrap().clone().clients;
        let mut ok_clients: HashMap<i32, SseUser> = HashMap::new();

        for (id, client) in clients {
            if client
                .sender
                .send(sse::Event::Comment("ping".into()))
                .await
                .is_ok()
            {
                ok_clients.insert(id, client.clone());
            } else {
                self._disconnect(&mut self.db.borrow_mut().acquire().await.unwrap(), id);
            }
        }

        self.inner.write().unwrap().clients = ok_clients;
    }

    async fn _disconnect(
        &self,
        db: &mut PoolConnection<Postgres>,
        id: i32
    ) -> () {

    }

    /// Registers client with broadcaster, returning an SSE response body.
    pub async fn new_client(
        &self,
        db: &mut PoolConnection<Postgres>,
        id: i32,
        email: String,
    ) -> Sse<ChannelStream> {
        let (sender, channel_stream) = sse::channel(10);

        sender.send(sse::Data::new("connected")).await.unwrap();
        println!("creating new clients success {:?}",sender);
        
        let index = self.inner.read().unwrap().index;
        let sse_user: SseUser = SseUser{
            _id: index,
            sender: sender,
            email: email,
            groups: Vec::new()
        };
        self.inner.write().unwrap().index += 1;
        self.inner.write().unwrap().clients.insert(id, sse_user);

        self._set_status_by_id(db, id, true, true);
        self._broadcast_status_update(id, true);
        channel_stream
    }

    /// Broadcasts `msg` to all clients.
    pub async fn broadcast(&self, msg: &str) {
        let clients = self.inner.write().unwrap().clients.clone();
        let send_futures = clients
            .iter()
            .map(|client| client.1.sender.send(sse::Data::new(msg)));

        // try to send to all clients, ignoring failures
        // disconnected clients will get swept up by `remove_stale_clients`
        let _ = future::join_all(send_futures).await;
    }

//----------------------------------------------------------------------------------
// lepsze funkcje niżej
//---------------------------------------------------------------------------------- 

    async fn _set_status_by_id(
        &self,
        db: &mut PoolConnection<Postgres>,
        userid: i32,
        isactive: bool,
        set_last_login: bool
    ) -> () {
        let mut conn = db.acquire().await.unwrap();
        if set_last_login {
            sqlx::query("UPDATE users SET status=$1, last_login=now() WHERE id=$2")
                .bind(isactive)
                .bind(userid)
                .execute(&mut *conn);
        } else {
            sqlx::query("UPDATE users SET status=$1 WHERE id=$2")
                .bind(isactive)
                .bind(userid)
                .execute(&mut *conn);
        }
    }

    async fn _set_status_by_sseuser(
        &self,
        db: &mut PoolConnection<Postgres>,
        user: &SseUser,
        isactive: bool,
        set_last_login: bool
    ) -> () {
        let mut conn = db.acquire().await.unwrap();
        let mut id = -1;
        for (idd, userr) in &self.inner.read().unwrap().clients {
            if userr._id == user._id {
                id = *idd;
            }
        };
        
        if set_last_login {
            sqlx::query("UPDATE users SET status=$1, last_login=now() WHERE id=$2")
                .bind(isactive)
                .bind(id)
                .execute(&mut *conn);
        } else {
            sqlx::query("UPDATE users SET status=$1 WHERE id=$2")
                .bind(isactive)
                .bind(id)
                .execute(&mut *conn);
        }
    }

    async fn _broadcast_status_update(
        &self,
        userid: i32,
        active: bool
    ) -> () {
        let user: Option<SseUser> = self.inner.read().unwrap().clients.get(&userid).cloned();

        if user.is_none() {
            return
        }

        let user = user.unwrap();
        let clients = self.inner.write().unwrap().clients.clone();
        let send_futures = clients
            .iter()
            .map(|client| client.1.sender.send(sse::Data::new(
                format!(
                    "{{\"type\":\"user_status_update\", \"user_email\":\"{}\", \"active\":{}}}",
                    user.email,
                    active
                )
            )));

        let _ = future::join_all(send_futures).await;
    }
}
