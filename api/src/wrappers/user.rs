use nanorand::{Rng, tls_rng};
// use chrono::{NaiveDate, Utc};

pub fn random_password(length: usize) -> String {
    let mut rng = tls_rng();
    let mut numbers: Vec<u32> = Vec::with_capacity(length);
    for _ in 0..length {
        numbers.push(rng.generate_range(65..90)); // A-Z
    }
    numbers
        .into_iter()
        .map(|d| std::char::from_u32(u32::from(d)).unwrap())
        .collect()
}

pub mod user_admin {
    use crate::structs::DbUser;
    use crate::utils::{DecupUnwrap, DecupUnwrapActix, errprint, ez};

    use actix_web::{Error, error};
    use chrono::NaiveDate;
    use futures_util::TryFutureExt;
    use sqlx::{Acquire, Postgres, pool::PoolConnection};

    use super::random_password;

    pub async fn get_group_ids(
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

    pub async fn get(db: &mut PoolConnection<Postgres>, id: i32) -> Result<DbUser, Error> {
        let conn = db.acquire().await.unwrap();
        let mut er: Option<Error> = None;
        let user: Option<DbUser> = match sqlx::query_as("SELECT * FROM users WHERE id=$1;")
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
        {
            Ok(a) => a,
            Err(sqlx::Error::RowNotFound) => {
                er = Some(error::ErrorBadRequest("Użytkownik nie istnieje."));
                None
            }
            Err(err) => {
                errprint!("{}", err);
                er = Some(error::ErrorInternalServerError(
                    "Wystąpił błąd.",
                ));
                None
            }
        };

        ez!(er);
        Ok(user.unwrap())
}

    pub async fn get_user_type_ids(
        db: &mut PoolConnection<Postgres>,
        userid: i32,
    ) -> Result<Vec<i32>, sqlx::Error> {
        let conn = db.acquire().await.unwrap();
        let mut er: Option<sqlx::Error> = None;
        let user_types: Option<Vec<i32>> =
            sqlx::query_scalar("SELECT user_type_id FROM users_users_type WHERE user_id = $1;")
                .bind(userid)
                .fetch_all(&mut *conn)
                .await
                .decup(&mut er, true);

        ez!(er);
        Ok(user_types.unwrap())
    }

    pub async fn add(
        db: &mut PoolConnection<Postgres>,
        first_name: &String,
        last_name: &String,
        email: &String,
        birth_date: &NaiveDate,
        user_types: &Vec<i32>,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = db.begin().await.unwrap();

        let mut er: Option<sqlx::Error> = None;
        let (id,): (i32,) = sqlx::query_as("
        INSERT INTO users (
            first_name,
            last_name,
            email,
            password,
            birth_date,
            last_login,
            bio
        ) VALUES ($1, $2, $3, $4, $5,
            now(),
            ''
        ) RETURNING id;",)
        .bind(first_name)
        .bind(last_name)
        .bind(email)
        .bind(random_password(12))
        .bind(birth_date)
        .fetch_one(&mut *transaction)
        .await
        .unwrap_or_else(|err| {
            errprint!("{}", err);
            er = Some(err);
            (-1,)
        });
        ez!(er);

        for i in user_types {
            sqlx::query(
                "
            INSERT INTO users_users_type (
                user_id,
                user_type_id
            ) VALUES ($1, $2)
            ",
            )
            .bind(id)
            .bind(&i)
            .execute(&mut *transaction)
            .await
            .unwrap();
        }

        transaction.commit().await.unwrap_or_else(|err| {
            errprint!("{}", err);
            er = Some(err);
        });

        ez!(er);
        Ok(())
    }

    pub async fn update(
        db: &mut PoolConnection<Postgres>,
        userid: i32,
        first_name: String,
        last_name: String,
        email: String,
        birth_date: NaiveDate,
        users_types: Vec<i32>
    ) -> Result<(), Error> {
        let conn = db.acquire().await.unwrap();
        let mut transaction = conn.begin().await.unwrap();

        let mut er: Option<Error> = None;
        _ = sqlx::query("
        UPDATE users SET
            first_name=$1,
            last_name=$2,
            email=$3,
            birth_date=$4
        WHERE id=$5;")
        .bind(first_name)
        .bind(last_name)
        .bind(email)
        .bind(birth_date)
        .bind(userid)
        .execute(&mut *transaction)
        .await
        .decup_actix(&mut er, true);
        
        if er.is_some() {
            transaction.rollback().await
                .map(|_| ()).unwrap_or_else(|err| {
                errprint!("{}", err);
                ()
            });
            return Err(error::ErrorInternalServerError("Wystąpił błąd."));
        }

        _ = sqlx::query("DELETE FROM users_users_type WHERE user_id={id};")
            .execute(&mut *transaction)
            .await
            .decup_actix(&mut er, true);

        if er.is_some() {
            transaction.rollback().await
                .map(|_| ()).unwrap_or_else(|err| {
                errprint!("{}", err);
                ()
            });
            return Err(error::ErrorInternalServerError("Wystąpił błąd."));
        }

        for utype in users_types {
            _ = sqlx::query("INSERT INTO users_users_type WHERE user_id=$1;")
            .bind(utype)
            .execute(&mut *transaction)
            .await
            .decup_actix(&mut er, true);
        }

        if er.is_some() {
            transaction.rollback().await
                .map(|_| ()).unwrap_or_else(|err| {
                errprint!("{}", err);
                ()
            });
            return Err(error::ErrorInternalServerError("Wystąpił błąd."));
        }

        transaction.commit().await.unwrap_or_else(|err| {
            errprint!("{}", err);
            er = Some(error::ErrorInternalServerError("Wystąpił błąd."));
        });

        ez!(er); Ok(())
    }

    pub async fn delete(
        db: &mut PoolConnection<Postgres>,
        userid: i32
    ) -> Result<(), Error> {
        let conn = db.acquire().await.unwrap();
        let mut transaction = conn.begin().await.unwrap();

        let mut er: Option<Error> = None;
        _ = sqlx::query("DELETE FROM group_members WHERE user_id=$1;")
            .bind(userid)
            .execute(&mut *transaction)
            .await
            .decup_actix(&mut er, true);

        if er.is_some() {
            transaction.rollback().await
                .map(|_| ()).unwrap_or_else(|err| {
                errprint!("{}", err);
                ()
            });
            return Err(error::ErrorInternalServerError("Wystąpił błąd."));
        }

        _ = sqlx::query("DELETE FROM users_users_type WHERE user_id=$1;")
            .bind(userid)
            .execute(&mut *transaction)
            .await
            .decup_actix(&mut er, true);

        if er.is_some() {
            transaction.rollback().await
                .map(|_| ()).unwrap_or_else(|err| {
                errprint!("{}", err);
                ()
            });
            return Err(error::ErrorInternalServerError("Wystąpił błąd."));
        }

        _ = sqlx::query("DELETE FROM users WHERE id=$1;")
            .bind(userid)
            .execute(&mut *transaction)
            .await
            .decup_actix(&mut er, true);

        if er.is_some() {
            transaction.rollback().await
                .map(|_| ()).unwrap_or_else(|err| {
                errprint!("{}", err);
                ()
            });
            return Err(error::ErrorInternalServerError("Wystąpił błąd."));
        }

        transaction.commit().await.unwrap_or_else(|err| {
            errprint!("{}", err);
            er = Some(error::ErrorInternalServerError("Wystąpił błąd."));
        });

        ez!(er); Ok(())
    }
}
