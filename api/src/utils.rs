use sqlx::PgConnection;
use std::io::Error;
use std::io::ErrorKind::Other;
macro_rules! errprint {
    () => {
        println!("[\x1b[31mERROR\x1b[0m / \x1b[33m{}\x1b[0m:\x1b[31m{}\x1b[0m]", fil!(), line!())
    };
    ($form:tt, $($arg:tt)*) => {{
        println!(
            concat!("[\x1b[31mERROR\x1b[0m / \x1b[33m{}\x1b[0m:\x1b[31m{}\x1b[0m] ", $form), file!(), line!(), $($arg)*
        )
    }};
}

macro_rules! warnprint {
    () => {
        println!("[\x1b[33mWARN \x1b[0m / \x1b[33m{}\x1b[0m:\x1b[31m{}\x1b[0m]", fil!(), line!())
    };
    ($form:tt, $($arg:tt)*) => {{
        println!(
            concat!("[\x1b[33mWARN \x1b[0m / \x1b[33m{}\x1b[0m:\x1b[31m{}\x1b[0m] ", $form), file!(), line!(), $($arg)*
        )
    }};
}

macro_rules! trans_multier {
    ($transaction:expr, $($sql:literal)*) => {
        $(
            _ = sqlx::query!($sql).fetch_all(&mut *$transaction).await.unwrap();
        )*
    }
}

pub(crate) use {errprint, trans_multier};

pub async fn trans_multi(sql: String, transaction: &mut PgConnection) -> Result<(), sqlx::Error> {
    let mut err_string = String::from("");
    let mut is_err = false;
    for line in sql.split(";") {
        match sqlx::query(line).execute(&mut *transaction).await {
            Ok(_) => {}
            Err(err) => {
                is_err = true;
                let err_msg = &format!("{}\n", err)[..];
                if err_msg
                    != "error returned from database: current transaction is aborted, commands ignored until end of transaction block\n"
                {
                    err_string += err_msg;
                }
            }
        }
    }

    if is_err {
        // return Err(sqlx::Error::Io(err_string));
        return Err(sqlx::Error::Io(Error::new(Other, err_string)));
    }
    Ok(())
}
