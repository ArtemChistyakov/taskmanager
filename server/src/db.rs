use std::fs;
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Utc};
use mobc::{Connection, Pool};
use mobc_postgres::PgConnectionManager;
use mobc_postgres::tokio_postgres::{Config, NoTls, Row};

use crate::{DBPool, error};
use common::data::{Pageable, Task, User, UserRequest};
use crate::error::Error::{DBInitError, DBPoolError, DBQueryError};

const INIT_SQL: &str = "./db.sql";
const USER_SELECT_FIELDS: &str = "id,first_name,last_name,email,created_at";
const USER_INSERT_FIELDS: &str = "first_name,last_name,email";
const USERS_TABLE_NAME: &str = "users";

const TASK_SELECT_FIELDS: &str = "id,title,project_id,created_at";
const TASKS_TABLE_NAME: &str = "tasks";

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
                        pageable.limit.unwrap_or(10),
                        pageable.offset.unwrap_or(0));
    let row_users = con.query(query.as_str(), &[]).await.map_err(DBQueryError)?;
    let users = row_users.iter()
        .map(|row| row_to_user(row))
        .collect::<Vec<_>>();
    Ok(users)
}

pub(crate) async fn find_tasks(db_pool: &DBPool, pageable: Pageable) -> Result<Vec<Task>> {
    let con = get_conn(db_pool).await?;
    let query = format!("SELECT {}  FROM {} {} ORDER BY {} {} LIMIT {} OFFSET {}",
                        TASK_SELECT_FIELDS, TASKS_TABLE_NAME, "",
                        pageable.order_by.unwrap_or("id".to_string()),
                        pageable.direction.unwrap_or("ASC".to_string()),
                        pageable.limit.unwrap_or(10),
                        pageable.offset.unwrap_or(0));
    let row_tasks = con.query(query.as_str(), &[]).await.map_err(DBQueryError)?;
    let tasks = row_tasks.iter().map(|row_task| row_to_task(row_task))
        .collect::<Vec<Task>>();
    Ok(tasks)
}

pub(crate) async fn create_user(db_pool: &DBPool, user_request: UserRequest) -> Result<User> {
    let con = get_conn(db_pool).await?;
    let query = format!("INSERT INTO {} ({}) VALUES ($1,$2,$3) RETURNING {}",
                        USERS_TABLE_NAME,
                        USER_INSERT_FIELDS,
                        USER_SELECT_FIELDS
    );
    let user_row = con
        .query_one(query.as_str(), &[&user_request.first_name,
            &user_request.last_name,
            &user_request.email])
        .await
        .map_err(DBQueryError)?;
    let user = row_to_user(&user_row);
    Ok(user)
}

fn row_to_user(row: &Row) -> User {
    let id: i32 = row.get(0);
    let first_name: Option<String> = row.get(1);
    let last_name: Option<String> = row.get(2);
    let email: Option<String> = row.get(3);
    let created_at: DateTime<Utc> = row.get(4);
    User {
        id,
        first_name,
        last_name,
        email,
        created_at,
    }
}

fn row_to_task(row: &Row) -> Task {
    let id: i32 = row.get(0);
    let title: String = row.get(1);
    let project_id: i32 = row.get(2);
    let created_at: DateTime<Utc> = row.get(3);
    Task {
        id,
        title,
        project_id,
        created_at,
    }
}