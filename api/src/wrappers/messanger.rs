use sqlx::{pool::PoolConnection, Acquire, Postgres};
use actix_web_lab::sse;
use futures_util::future;

use crate::{structs::SerDeSer, utils::errprint};

use super::eventor::{self, Eventor};

pub async fn send(
    eventor: &Eventor,
    message: String,
    senderid: i32,
    recipientid: Option<i32>,
    groupid: Option<i32>,

) -> () {
    let mut conn = eventor.db.acquire().await.unwrap();
    _ = sqlx::query("
        INSERT INTO messages (
            sender_id,
            recipient_id,
            group_id,
            content,
            group_name
        ) VALUES ($1, $2, $3, $4, $5);
    ")
    .bind(senderid)
    .bind(recipientid)
    .bind(groupid)
    .bind(&message)
    .bind("erm")
    .execute(&mut *conn)
    .await;

    if groupid.is_some() {
        let groupid = groupid.unwrap();
        let clients = eventor.inner.read().unwrap().clients.clone();
        let send_futures = clients
            .iter()
            .filter(|client| {
                client.groups.contains(&groupid)
            })
            .map(|client| {
                client.sender.send(sse::Data::new(format!(
                    "{{
                        \"type\":\"received_group_message\",
                        \"message\":\"{}\",
                        \"group\":\"{}\"
                    }}",
                    message.clone().ser(),
                    groupid
                )))
            });

        let _ = future::join_all(send_futures).await;
    } else if recipientid.is_some() {
        let recipientid = recipientid.unwrap();
        let clients = eventor.inner.read().unwrap().clients.clone();
        let send_futures = clients
            .iter()
            .filter(|client| {
                client.id == recipientid
            })
            .map(|client| {
                client.sender.send(sse::Data::new(format!(
                    "{{
                        \"type\":\"received_dm\",
                        \"message\":\"{}\",
                        \"sender\":\"{}\"
                    }}",
                    message.clone().ser(),
                    recipientid
                )))
            });

        let _ = future::join_all(send_futures).await;
    }
}

pub async fn get_receivers(
    db: &mut PoolConnection<Postgres>,
    messageid: i32
) -> (Option<i32>, Option<i32>) {
    let conn = db.acquire().await.unwrap();
    let receivers: (Option<i32>, Option<i32>) = sqlx::query_as("SELECT recipient_id, group_id FROM messages WHERE id=$1 LIMIT 1;")
        .bind(messageid)
        .fetch_one(&mut *conn)
        .await
        .unwrap_or_else(|err| {
            errprint!("{}", err);
            (None, None)
        });
    receivers
}

pub async fn get_sender(
    db: &mut PoolConnection<Postgres>,
    messageid: i32
) -> Option<i32> {
    let conn = db.acquire().await.unwrap();
    let sender: Option<i32> = sqlx::query_scalar("SELECT sender_id FROM messages WHERE id=$1 LIMIT 1;")
        .bind(messageid)
        .fetch_one(&mut *conn)
        .await
        .unwrap_or_else(|err| {
            errprint!("{}", err);
            None
        });
    sender
}

pub async fn edit(
    eventor: &Eventor,
    messageid: i32,
    new_content: String,
    pot_sender: i32,
    validate: bool
) -> () {
    let mut conn = eventor.db.acquire().await.unwrap();
    if validate {
        _ = sqlx::query("UPDATE messages SET content=$1 WHERE messageid=$2 AND sender_id=$3;")
            .bind(&new_content)
            .bind(&messageid)
            .bind(pot_sender)
            .execute(&mut *conn)
            .await
    } else {
        _ = sqlx::query("UPDATE messages SET content=$1 WHERE messageid=$2;")
            .bind(&new_content)
            .bind(&messageid)
            .execute(&mut *conn)
            .await
    }

    let (groupid, recipientid) = get_receivers(&mut conn, messageid).await;

    if groupid.is_some() {
        let groupid = groupid.unwrap();
        let clients = eventor.inner.read().unwrap().clients.clone();
        let send_futures = clients
            .iter()
            .filter(|client| {
                client.groups.contains(&groupid)
            })
            .map(|client| {
            client.sender.send(sse::Data::new(format!(
                "{{
                    \"type\":\"edited_group_message\",
                    \"group_id\":{},
                    \"message_id\":{},
                    \"new_content\":{}
                }}",
                groupid,
                messageid,
                new_content
            )))
        });

        let _ = future::join_all(send_futures).await;
    } else if recipientid.is_some() {
        let recipientid = recipientid.unwrap();
        let clients = eventor.inner.read().unwrap().clients.clone();
        let send_futures = clients
            .iter()
            .filter(|client| {
                client.id == recipientid
            })
            .map(|client| {
            client.sender.send(sse::Data::new(format!(
                "{{
                    \"type\":\"edited_dm\",
                    \"message_id\":{},
                    \"new_content\":{}
                }}",
                messageid,
                new_content
            )))
        });

        let _ = future::join_all(send_futures).await;
    }
}

pub async fn delete(
    eventor: &Eventor,
    messageid: i32,
    pot_sender: i32,
    validate: bool
) -> () {
    let mut conn = eventor.db.acquire().await.unwrap();
    if validate {
        _ = sqlx::query("DELETE FROM messages WHERE id=$1 AND sender_id=$2;")
            .bind(&messageid)
            .bind(pot_sender)
            .execute(&mut *conn)
            .await
    } else {
        _ = sqlx::query("DELETE FROM messages WHERE id=$1;")
            .bind(&messageid)
            .execute(&mut *conn)
            .await
    }

    let (groupid, recipientid) = get_receivers(&mut conn, messageid).await;

    if groupid.is_some() {
        let groupid = groupid.unwrap();
        let clients = eventor.inner.read().unwrap().clients.clone();
        let send_futures = clients
            .iter()
            .filter(|client| {
                client.groups.contains(&groupid)
            })
            .map(|client| {
            client.sender.send(sse::Data::new(format!(
                "{{
                    \"type\":\"deleted_group_message\",
                    \"group_id\":{},
                    \"message_id\":{},
                }}",
                groupid,
                messageid,
            )))
        });

        let _ = future::join_all(send_futures).await;
    } else if recipientid.is_some() {
        let recipientid = recipientid.unwrap();
        let clients = eventor.inner.read().unwrap().clients.clone();
        let send_futures = clients
            .iter()
            .filter(|client| {
                client.id == recipientid
            })
            .map(|client| {
            client.sender.send(sse::Data::new(format!(
                "{{
                    \"type\":\"deleted_dm\",
                    \"message_id\":{},
                }}",
                messageid,
            )))
        });

        let _ = future::join_all(send_futures).await;
    }
}
