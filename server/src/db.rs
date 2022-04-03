use std::fs;
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Utc};
use mobc::{Connection, Pool};
use mobc_postgres::PgConnectionManager;
use mobc_postgres::tokio_postgres::{Config, NoTls, Row};

use common::data::{CreateProjectParam, LoginRequest, Pageable, Project, ProjectRequest, Task, TaskRequest, User, UserRequest};

use crate::{DBPool, error};
use crate::error::Error;
use crate::error::Error::{DBInitError, DBPoolError, DBQueryError};

const INIT_SQL: &str = "./db.sql";
const USER_SELECT_FIELDS: &str = "id,first_name,last_name,email,created_at";
const USER_INSERT_FIELDS: &str = "first_name,last_name,email";
const TASK_INSERT_FIELDS: &str = "title,project_id";
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

pub async fn find_user_by_email_and_pwd(db_pool: DBPool, login_request: LoginRequest) -> Result<User> {
    let con = get_conn(&db_pool).await?;
    let query = format!("SELECT {}  FROM {} where email = $1 and password = $2",
                        USER_SELECT_FIELDS, USERS_TABLE_NAME
    );
    let opt_row = Some(con
        .query_one(query.as_str(), &[&login_request.email, &login_request.pwd])
        .await.map_err(DBQueryError)?);
    match opt_row {
        Some(row) => {
            Ok(row_to_user(&row))
        }
        None => Err(Error::WrongCredentialsError)
    }
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

pub async fn create_task(db_pool: DBPool, task_request: TaskRequest) -> Result<Task> {
    let con = get_conn(&db_pool).await?;
    let query = format!("INSERT INTO {} ({}) VALUES ($1,$2) RETURNING {}",
                        TASKS_TABLE_NAME,
                        TASK_INSERT_FIELDS,
                        TASK_SELECT_FIELDS
    );
    let task_row = con.query_one(query.as_str(),
                                 &[&task_request.title, &task_request.project_id])
        .await
        .map_err(DBQueryError)?;
    let task = row_to_task(&task_row);
    Ok(task)
}

pub async fn create_project(db_pool: DBPool, param: CreateProjectParam, project_request: ProjectRequest) -> Result<Project> {
    let con = get_conn(&db_pool).await?;
    let query = format!("INSERT INTO {} ({}) VALUES ($1) RETURNING {}",
                        "projects",
                        "title",
                        "id,title,created_at"
    );
    let project_row = con.query_one(query.as_str(),
                                    &[&project_request.title])
        .await
        .map_err(DBQueryError)?;
    let project = row_to_project(&project_row);
    create_user_project_reference(&db_pool, param.user_id, project.id).await?;
    Ok(project)
}

pub async fn create_user_project_reference(db_pool: &DBPool, user_id: i32, project_id: i32) -> Result<()> {
    let con = get_conn(&db_pool).await?;
    let query = format!("INSERT INTO {} ({}) VALUES ($1,$2)",
                        "users_projects",
                        "user_id,project_id"
    );
    let _number_of_rows_modified = con.execute(query.as_str(),
                                               &[&user_id, &project_id])
        .await
        .map_err(DBQueryError)?;
    Ok(())
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

fn row_to_project(row: &Row) -> Project {
    let id: i32 = row.get(0);
    let title: String = row.get(1);
    let created_at: DateTime<Utc> = row.get(2);
    Project {
        id,
        title,
        created_at,
    }
}