use std::time::Duration;

use bcrypt::DEFAULT_COST;
use chrono::{DateTime, Utc};
use mobc::{Connection, Pool};
use mobc_postgres::PgConnectionManager;
use mobc_postgres::tokio_postgres::{Config, NoTls, Row, Transaction};
use refinery::config::ConfigDbType;
use uuid::Uuid;

use common::data::{Pageable, Project, ProjectRequest, Task, TaskRequest, User, UserRequest};

use crate::{DBPool, embedded, error};
use crate::data::VerificationToken;
use crate::error::Error;
use crate::error::Error::{DBInitError, DBInitErrorTest, DBPoolError, DBQueryError, EncryptPasswordError, WrongCredentialsError};

const USER_SELECT_FIELDS: &str = "id,first_name,last_name,email,pwd,ebabled,created_at";
const USER_INSERT_FIELDS: &str = "first_name,last_name,email,pwd";
const USERS_TABLE_NAME: &str = "app_users";

const TOKENS_SELECT_FIELDS: &str = "id,user_id,token,expiry_date";
const TOKENS_INSERT_FIELDS: &str = "user_id,token,expiry_date";
const TOKENS_TABLE_NAME: &str = "verification_tokens";

const TASK_SELECT_FIELDS: &str = "id,title,description,user_id,project_id,created_at";
const TASKS_TABLE_NAME: &str = "tasks";
const TASK_INSERT_FIELDS: &str = "title,description,user_id,project_id";

const PROJECT_SELECT_FIELDS: &str = "id,title,description,created_at";
const PROJECT_TABLE_NAME: &str = "projects";
const PROJECT_INSERT_FIELDS: &str = "title,description";

const USERS_PROJECTS_TABLE_NAME: &str = "users_projects";

type DBCon = Connection<PgConnectionManager<NoTls>>;
type Result<T> = std::result::Result<T, error::Error>;

pub fn create_pool(config: &crate::config::Config) -> Result<DBPool> {
    let mut pool_config = Config::new();
    pool_config.password(&config.postgres_password)
        .user(&config.postgres_password)
        .host(&config.postgres_host)
        .port(config.postgres_port)
        .dbname(&config.dbname);

    let manager = PgConnectionManager::new(pool_config, NoTls);
    Ok(
        Pool::builder()
            .max_open(20)
            .max_idle(8)
            .get_timeout(Some(Duration::from_secs(15)))
            .build(manager)
    )
}


pub async fn db_init(config: &crate::config::Config) -> Result<()> {
    let mut init_config = refinery::config::Config::new(ConfigDbType::Postgres)
        .set_db_host(&config.postgres_host)
        .set_db_name(&config.dbname)
        .set_db_port(&config.postgres_port.to_string())
        .set_db_user(&config.postgres_username)
        .set_db_pass(&config.postgres_password);

    embedded::migrations::runner()
        .run_async(&mut init_config)
        .await
        .map_err(|x| {
            println!("{:?}", x);
            DBInitErrorTest
        })?;
    Ok(())
}

pub async fn get_conn(db_pool: &DBPool) -> Result<DBCon> {
    db_pool.get().await.map_err(DBPoolError)
}


pub async fn find_user_by_email(db_pool: DBPool, email: &str) -> Result<User> {
    let con = get_conn(&db_pool).await?;
    let query = format!("SELECT {}  FROM {} where email = $1",
                        USER_SELECT_FIELDS, USERS_TABLE_NAME
    );

    let row = con
        .query_one(query.as_str(), &[&email])
        .await
        .map_err(|_| WrongCredentialsError)?;
    Ok(row_to_user(&row))
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

pub(crate) async fn find_tasks(db_pool: &DBPool, pageable: Pageable, user_id: i32) -> Result<Vec<Task>> {
    let con = get_conn(db_pool).await?;
    let query = get_select_query(TASK_SELECT_FIELDS,
                                 TASKS_TABLE_NAME,
                                 "WHERE user_id = $1",
                                 pageable);
    let row_tasks = con.query(query.as_str(), &[&user_id])
        .await
        .map_err(DBQueryError)?;
    let tasks = row_tasks.iter().map(|row_task| row_to_task(row_task))
        .collect::<Vec<Task>>();
    Ok(tasks)
}


fn get_select_query(select_fields: &str,
                    table_name: &str,
                    where_clause: &str,
                    pageable: Pageable) -> String {
    format!("SELECT {}  FROM {} {} ORDER BY {} {} LIMIT {} OFFSET {}",
            select_fields,
            table_name,
            where_clause,
            pageable.order_by.unwrap_or("id".to_string()),
            pageable.direction.unwrap_or("ASC".to_string()),
            pageable.limit.unwrap_or(10),
            pageable.offset.unwrap_or(0))
}

pub(crate) async fn find_projects(db_pool: &DBPool,
                                  pageable: Pageable,
                                  user_id: i32) -> Result<Vec<Project>> {
    let con = get_conn(db_pool).await?;
    let query = get_select_query("p.id,p.title,p.description,p.created_at",
                                 "projects p JOIN users_projects up ON p.id = up.project_id",
                                 "WHERE up.user_id = $1",
                                 pageable);
    let row_tasks = con.query(query.as_str(), &[&user_id])
        .await
        .map_err(DBQueryError)?;
    let projects = row_tasks.iter().map(|row_project| row_to_project(row_project))
        .collect::<Vec<Project>>();
    Ok(projects)
}

pub(crate) async fn create_user_and_verification_token(db_pool: &DBPool,
                                                       user_request: UserRequest) -> Result<(User, VerificationToken)> {
    let create_user_query = format!("INSERT INTO {} ({}) VALUES ($1,$2,$3,$4) RETURNING {}",
                                    USERS_TABLE_NAME,
                                    USER_INSERT_FIELDS,
                                    USER_SELECT_FIELDS
    );
    let create_token_query =
        format!("INSERT INTO {} ({})  VALUES ($1,$2,$3) RETURNING {}",
                TOKENS_TABLE_NAME,
                TOKENS_INSERT_FIELDS,
                TOKENS_SELECT_FIELDS);
    let encrypted_pwd = bcrypt::hash(user_request.pwd, DEFAULT_COST)
        .map_err(EncryptPasswordError)?;
    let mut con = get_conn(db_pool).await?;
    let transaction = con.transaction().await
        .map_err(DBQueryError)?;
    let user_row = transaction
        .query_one(create_user_query.as_str(), &[&user_request.first_name,
            &user_request.last_name,
            &user_request.email,
            &encrypted_pwd])
        .await
        .map_err(DBQueryError);
    if let Err(e) = user_row {
        transaction.rollback();
        return Err(e);
    }
    let user_row = user_row.unwrap();
    let user = row_to_user(&user_row);
    let expiry_date = chrono::offset::Utc::now() + chrono::Duration::minutes(30);
    let token_value = Uuid::new_v4().to_string();
    let verification_token_row = transaction.query_one(create_token_query.as_str(),
                                                       &[&user.id, &token_value, &expiry_date]).await;
    if let Err(e) = verification_token_row {
        transaction.rollback().await.map_err(DBQueryError)?;
        return Err(DBQueryError(e));
    }
    let verification_token_row = verification_token_row.unwrap();
    let verification_token = row_to_token(&verification_token_row);
    transaction.commit().await.map_err(DBQueryError)?;
    Ok((user, verification_token))
}

pub async fn create_task(db_pool: DBPool, task_request: TaskRequest,
                         user_id: i32) -> Result<Task> {
    let con = get_conn(&db_pool).await?;
    let query = format!("INSERT INTO {} ({}) VALUES ($1,$2,$3,$4) RETURNING {}",
                        TASKS_TABLE_NAME,
                        TASK_INSERT_FIELDS,
                        TASK_SELECT_FIELDS
    );
    let task_row = con.query_one(query.as_str(),
                                 &[&task_request.title,
                                     &task_request.description,
                                     &user_id,
                                     &task_request.project_id])
        .await
        .map_err(DBQueryError)?;
    let task = row_to_task(&task_row);
    Ok(task)
}


pub async fn create_project(db_pool: DBPool,
                            project_request: ProjectRequest,
                            user_id: i32) -> Result<Project> {
    let mut connection = get_conn(&db_pool).await?;
    let transaction = connection.transaction()
        .await
        .map_err(DBQueryError)?;
    let query = format!("INSERT INTO {} ({}) VALUES ($1,$2) RETURNING {}",
                        PROJECT_TABLE_NAME,
                        PROJECT_INSERT_FIELDS,
                        PROJECT_SELECT_FIELDS
    );
    let project_row = transaction.query_one(query.as_str(),
                                            &[&project_request.title,
                                                &project_request.description])
        .await
        .map_err(DBQueryError)?;
    let project = row_to_project(&project_row);
    let res = create_user_project_reference(&transaction, user_id, project.id)
        .await;
    if let Err(_) = res {
        transaction.rollback()
            .await
            .map_err(DBQueryError)?;
        return Err(res.err().unwrap());
    }
    transaction.commit()
        .await
        .map_err(DBQueryError)?;
    Ok(project)
}

pub async fn create_user_project_reference(transaction: &Transaction<'_>, user_id: i32, project_id: i32) -> Result<()> {
    let query = format!("INSERT INTO {} ({}) VALUES ($1,$2)",
                        "users_projects",
                        "user_id,project_id"
    );
    transaction.execute(query.as_str(), &[&user_id, &project_id])
        .await
        .map_err(DBQueryError)?;
    Ok(())
}


pub(crate) async fn delete_task(db_pool: DBPool, task_id: i32, user_id: i32) -> Result<u64> {
    let con = get_conn(&db_pool).await?;
    let query = format!("DELETE FROM {} \
     WHERE id = $1", TASKS_TABLE_NAME);
    con.execute(query.as_str(), &[&task_id])
        .await
        .map_err(DBQueryError)
}

pub(crate) async fn delete_project(db_pool: DBPool, project_id: i32, user_id: i32) -> Result<u64> {
    let mut con = get_conn(&db_pool).await?;
    let transaction = con.transaction().await.map_err(DBQueryError)?;
    let query = format!("DELETE FROM {} \
     WHERE user_id = $1 AND project_id= $2", USERS_PROJECTS_TABLE_NAME);
    let res = transaction.execute(
        query.as_str(),
        &[&user_id, &project_id])
        .await;
    if let Err(e) = res {
        transaction.rollback().await
            .map_err(DBQueryError)?;
        return Err(DBQueryError(e));
    }
    let query = format!("DELETE FROM {} \
     WHERE id = $1", PROJECT_TABLE_NAME);
    let res = transaction.execute(query.as_str(), &[&project_id])
        .await;
    return match res {
        Err(e) => {
            println!("{:?}", e);
            transaction.rollback().await
                .map_err(DBQueryError)?;
            Err(DBQueryError(e))
        }
        Ok(row_count) => {
            transaction.commit().await
                .map_err(DBQueryError)?;
            Ok(row_count)
        }
    };
}

fn row_to_user(row: &Row) -> User {
    let id: i32 = row.get(0);
    let first_name: Option<String> = row.get(1);
    let last_name: Option<String> = row.get(2);
    let email: String = row.get(3);
    let pwd: String = row.get(4);
    let enabled: bool = row.get(5);
    let created_at: DateTime<Utc> = row.get(6);
    User {
        id,
        first_name,
        last_name,
        email,
        pwd,
        enabled,
        created_at,
    }
}

fn row_to_task(row: &Row) -> Task {
    let id: i32 = row.get(0);
    let title: String = row.get(1);
    let description: Option<String> = row.get(2);
    let user_id: i32 = row.get(3);
    let project_id: i32 = row.get(4);
    let created_at: DateTime<Utc> = row.get(5);
    Task {
        id,
        title,
        description,
        user_id,
        project_id,
        created_at,
    }
}

fn row_to_project(row: &Row) -> Project {
    let id: i32 = row.get(0);
    let title: String = row.get(1);
    let description: Option<String> = row.get(2);
    let created_at: DateTime<Utc> = row.get(3);
    Project {
        id,
        title,
        description,
        created_at,
    }
}

fn row_to_token(row: &Row) -> VerificationToken {
    let id: i32 = row.get(0);
    let user_id: i32 = row.get(1);
    let token: String = row.get(2);
    let expiry_date: DateTime<Utc> = row.get(3);
    VerificationToken {
        id,
        user_id,
        token,
        expiry_date,
    }
}
