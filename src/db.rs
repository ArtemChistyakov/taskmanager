use std::fs;
use std::str::FromStr;
use std::time::Duration;

use mobc::{Connection, Pool};
use mobc_postgres::PgConnectionManager;
use mobc_postgres::tokio_postgres::{Config, NoTls};

use crate::{ DBPool, error};
use crate::data::{Pageable, Task, User};
use crate::error::Error::{DBInitError, DBPoolError};

const INIT_SQL: &str = "./db.sql";

type DBCon = Connection<PgConnectionManager<NoTls>>;
type Result<T> = std::result::Result<T, error::Error>;

pub fn create_pool() -> Result<DBPool> {
    let config = Config::from_str("postgres://postgres@127.0.0.1:5432/postgres")?;
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
pub async fn get_conn(db_pool: &DBPool)->Result<DBCon>{
    db_pool.get().await.map_err(DBPoolError)
}
pub(crate) async fn find_users(db_pool: &DBPool, pageable: &Pageable) -> Result<Vec<User>> {
    let con = db_pool.get().await.unwrap();
    todo!()
}

pub(crate) async fn find_tasks(p0: &DBPool, p1: &Pageable) -> Result<Vec<Task>> {
    todo!()
}