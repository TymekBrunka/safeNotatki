use std::{collections::HashMap, sync::{Arc, RwLock}, time::Duration};
use actix_web::rt::time::interval;
use actix_web_lab::sse::{self, ChannelStream, Sse};
use futures_util::future;
use sqlx::{pool::PoolConnection, Acquire, Postgres};
// use parking_lot::Mutex;

use crate::utils::{ez, DecupUnwrap};
use crate::structs::SseUser;
use crate::utils::sucprint;

pub struct Eventor {
    inner: RwLock<EventorData>,
}

#[derive(Debug, Clone)]
struct EventorData {
    clients: HashMap<i32, SseUser>
}

impl Eventor {
    /// Constructs new broadcaster and spawns ping loop.
    pub fn create() -> Arc<Self> {
        let this = Arc::new(Eventor {
            inner: RwLock::new(
                EventorData{
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
            }
        }

        self.inner.write().unwrap().clients = ok_clients;
    }

    /// Registers client with broadcaster, returning an SSE response body.
    pub async fn new_client(&self, id: i32, email: String) -> Sse<ChannelStream> {
        let (sender, channel_stream) = sse::channel(10);

        sender.send(sse::Data::new("connected")).await.unwrap();
        println!("creating new clients success {:?}",sender);
        
        let sse_user: SseUser = SseUser{
            sender: sender,
            email: email,
            groups: Vec::new()
        };
        self.inner.write().unwrap().clients.insert(id, sse_user);
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
}
