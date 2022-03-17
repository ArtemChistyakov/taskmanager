use std::fs;
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Utc};
use mobc::{Connection, Pool};
use mobc_postgres::PgConnectionManager;
use mobc_postgres::tokio_postgres::{Config, NoTls, Row};

use crate::{DBPool, error};
use crate::data::{Pageable, Task, User};
use crate::error::Error::{DBInitError, DBPoolError, DBQueryError};

const INIT_SQL: &str = "./db.sql";
const USER_SELECT_FIELDS: &str = "id,first_name,last_name,email,created_at";
const USERS_TABLE_NAME: &str = "users";

type DBCon = Connection<PgConnectionManager<NoTls>>;
type Result<T> = std::result::Result<T, error::Error>;

pub fn create_pool() -> Result<DBPool> {
    let mut config = Config::from_str("postgres://postgres@127.0.0.1:5432/postgres")?;
    config.user("admin");
    config.password("admin");
    let manager = PgConnectionManager::new(config, NoTls);
    Ok(
        Pool::builder()
            .max_open(20)
            .max_idle(8)
            .get_timeout(Some(Duration::from_secs(15)))
            .build(manager)
    )
}

pub async fn db_init(db_pool: &DBPool) -> Result<()> {
    let init_sql = fs::read_to_string(INIT_SQL)?;
    let connection = db_pool.get().await?;
    connection.batch_execute(init_sql.as_str())
        .await
        .map_err(DBInitError)?;
    Ok(())
}

pub async fn get_conn(db_pool: &DBPool) -> Result<DBCon> {
    db_pool.get().await.map_err(DBPoolError)
}

pub(crate) async fn find_users(db_pool: &DBPool, pageable: Pageable) -> Result<Vec<User>> {
    let con = get_conn(db_pool).await?;
    let query = format!("SELECT {}  FROM {} {} ORDER BY {} {} LIMIT {} OFFSET {}",
                        USER_SELECT_FIELDS, USERS_TABLE_NAME, "",
                        pageable.order_by.unwrap_or("id".to_string()),
                        pageable.direction.unwrap_or("ASC".to_string()),
                        pageable.limit.unwrap_or(0),
                        pageable.limit.unwrap_or(10));
    let row_users = con.query(query.as_str(), &[]).await.map_err(DBQueryError)?;
    let users = row_users.iter()
        .map(|row| row_to_user(row))
        .collect::<Vec<_>>();
    Ok(users)
}

pub(crate) async fn find_tasks(p0: &DBPool, p1: Pageable) -> Result<Vec<Task>> {
    todo!()
}

fn row_to_user(row: &Row) -> User {
    let id: i32 = row.get(0);
    let first_name: String = row.get(1);
    let last_name: String = row.get(2);
    let email: String = row.get(3);
    let created_at: DateTime<Utc> = row.get(4);
    User {
        id,
        first_name,
        last_name,
        email,
        created_at,
    }
}