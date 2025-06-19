use actix_web::rt::time::interval;
use actix_web_lab::sse::{self, ChannelStream, Sse};
use futures_util::future;
use sqlx::{Acquire, Pool, Postgres, pool::PoolConnection};
use std::{
    cell::RefCell, collections::HashMap, ops::Index, sync::{Arc, RwLock}, time::Duration
};
// use parking_lot::Mutex;

use crate::structs::SseUser;
use crate::utils::sucprint;
use crate::utils::{DecupUnwrap, ez, warnprint};

async fn get_group_ids(
    db: &mut PoolConnection<Postgres>,
    userid: i32,
) -> Result<Vec<i32>, sqlx::Error> {
    let conn = db.acquire().await.unwrap();
    let mut er: Option<sqlx::Error> = None;
    let ids: Option<Vec<i32>> = sqlx::query_scalar("SELECT id FROM groups JOIN group_members ON groups.id = group_members.group_id WHERE group_members.user_id = $1;")
        .bind(userid)
        .fetch_all(&mut *conn)
        .await
        .decup(&mut er, true);

    ez!(er);
    Ok(ids.unwrap())
}

pub struct Eventor {
    pub db: Pool<Postgres>,
    pub inner: RwLock<EventorData>,
}

#[derive(Debug)]
pub struct EventorData {
    index: i32,
    pub clients: Vec<SseUser>,
}

impl Eventor {
    /// Constructs new broadcaster and spawns ping loop.
    pub fn create(db: Pool<Postgres>) -> Arc<Self> {
        let this = Arc::new(Eventor {
            db: db,
            inner: RwLock::new(EventorData {
                index: 0,
                clients: Vec::new(),
            }),
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

    async fn _set_status_by_id(&self, userid: i32, isactive: bool, set_last_login: bool) -> () {
        let mut conn = self.db.acquire().await.unwrap();
        if set_last_login {
            _ = sqlx::query("UPDATE users SET status=$1, last_login=now() WHERE id=$2")
                .bind(isactive)
                .bind(userid)
                .execute(&mut *conn)
                .await;
        } else {
            _ = sqlx::query("UPDATE users SET status=$1 WHERE id=$2")
                .bind(isactive)
                .bind(userid)
                .execute(&mut *conn)
                .await;
        }
    }

    async fn _broadcast_status_update(&self, userid: i32, active: bool) -> () {
        let user: Option<SseUser> = self
            .inner
            .read()
            .unwrap()
            .clients
            .iter()
            .find(|userr| userr.id == userid)
            .cloned();

        if user.is_none() {
            return;
        }

        let user = user.unwrap();
        let clients = self.inner.write().unwrap().clients.clone();
        let send_futures = clients
            .iter()
            .map(|client| {
            client.sender.send(sse::Data::new(format!(
                "{{
                    \"type\":\"user_status_update\",
                    \"user_email\":\"{}\",
                    \"active\":{}
                }}",
                user.email, active
            )))
        });

        let _ = future::join_all(send_futures).await;
    }

    async fn _disconnect(&self, id: i32) -> () {
        self._set_status_by_id(id, true, false).await;
        self._broadcast_status_update(id, false).await;
    }

    /// Removes all non-responsive clients from broadcast list.
    async fn remove_stale_clients(&self) {
        let clients = self.inner.read().unwrap().clients.clone();
        let mut ok_clients: Vec<SseUser> = Vec::new();

        for client in clients {
            if client
                .sender
                .send(sse::Event::Comment("ping".into()))
                .await
                .is_ok()
            {
                ok_clients.push(client.clone());
            } else {
                self._disconnect(client.id).await;
            }
        }

        self.inner.write().unwrap().clients = ok_clients;
    }

    pub async fn close_and_disconnect(&self, userid: i32, do_set_status: bool) -> () {
        if do_set_status {
            self._set_status_by_id(userid, true, false).await;
        }
        self._broadcast_status_update(userid, false).await;
        
        let clients = self.inner.read().unwrap().clients.clone();
        let mut final_clients: Vec<SseUser> = Vec::new();

        for client in clients {
            if client.id == userid {
                final_clients.push(client);
            }
        }

        self.inner.write().unwrap().clients = final_clients;
    }

    /// Registers client with broadcaster, returning an SSE response body.
    pub async fn new_client(&self, id: i32, email: String) -> Sse<ChannelStream> {
        let (sender, channel_stream) = sse::channel(10);

        sender.send(sse::Data::new("connected")).await.unwrap();
        println!("creating new clients success {:?}", sender);

        let index = self.inner.read().unwrap().index;
        let sse_user: SseUser = SseUser {
            id: id,
            index: index,
            sender: sender,
            email: email,
            groups: Vec::new(),
        };
        self.inner.write().unwrap().index += 1;
        self.inner.write().unwrap().clients.push(sse_user);

        self._set_status_by_id(id, true, true).await;
        self._broadcast_status_update(id, true).await;
        channel_stream
    }

    // /// Broadcasts `msg` to all clients.
    // pub async fn broadcast(&self, msg: &str) {
    //     let clients = self.inner.write().unwrap().clients.clone();
    //     let send_futures = clients
    //         .iter()
    //         .map(|client| client.sender.send(sse::Data::new(msg)));
    //
    //     // try to send to all clients, ignoring failures
    //     // disconnected clients will get swept up by `remove_stale_clients`
    //     let _ = future::join_all(send_futures).await;
    // }

    // async fn _set_status_by_sseuser(
    //     &self,
    //     user: &SseUser,
    //     isactive: bool,
    //     set_last_login: bool
    // ) -> () {
    //     // let mut conn = db.acquire().await.unwrap();
    //     let mut conn = self.db.acquire().await.unwrap();
    //     let mut id = -1;
    //     for userr in &self.inner.read().unwrap().clients {
    //         if userr.index == user.index {
    //             id = user.id;
    //         }
    //     };
    //
    //     if set_last_login {
    //         _ = sqlx::query("UPDATE users SET status=$1, last_login=now() WHERE id=$2")
    //             .bind(isactive)
    //             .bind(id)
    //             .execute(&mut *conn)
    //             .await;
    //     } else {
    //         _ = sqlx::query("UPDATE users SET status=$1 WHERE id=$2")
    //             .bind(isactive)
    //             .bind(id)
    //             .execute(&mut *conn)
    //             .await;
    //     }
    // }

// ------------------------------ getery

    fn is_client_connected(&self, client: SseUser) -> bool {
        let mut ret = false;
        for clientt in self.inner.read().unwrap().clients.iter() {
            if clientt.index == client.index {
                ret = true;
                break;
            }
        }
        ret
    }

    fn is_user_connected(&self, userid: i32) -> bool {
        let mut ret = false;
        for clientt in self.inner.read().unwrap().clients.iter() {
            if clientt.id == userid {
                ret = true;
                break;
            }
        }
        ret
    }

}
