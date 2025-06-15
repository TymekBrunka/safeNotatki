use nanorand::{tls_rng, Rng};

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
    use crate::utils::errprint;
    use sqlx::{Pool, Postgres};

    use super::random_password;

    pub async fn add(db: Pool<Postgres>, first_name: String, last_name: String, email: String, birth_date: String, user_types: Vec<i32>) -> Result<(), sqlx::Error> {
        let mut conn = db;
        let mut transaction = conn.begin().await.unwrap();

        let er: Option<sqlx::Error> = None;
        let id: i32 = match sqlx::query_as("
        INSERT INTO users (
            first_name,
            last_name,
            email,
            password,
            birth_date,
            last_login,
            bio
        ) VALUES ($1, $2, $3, $4, $5
            now(),
            ''
        ) RETURNING id;")
            .bind(first_name)
            .bind(last_name)
            .bind(email)
            .bind(random_password(12))
            .bind(birth_date)
            .fetch_one(&mut *transaction)
            .await {
            Ok(a) => {a},
            Err(err) => {
                errprint!("{}", err);
                er = Some(err);
                -1
            }
        };

        if er.is_some() {
            return Err(er.unwrap());
        }

        for i in user_types {
            sqlx::query("
            INSERT INTO users_users_type (
                user_id,
                user_type_id
            ) VALUES ($1, $2)
            ")
            .bind(id)
            .bind(&i)
            .execute(&mut *transaction)
            .await
            .unwrap();
        }
        transaction.commit().await.unwrap_or_else(|err| {errprint!("{}", err)});

        Ok(())
    }
}
