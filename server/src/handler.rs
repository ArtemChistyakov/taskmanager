use warp::{reject, Reply};
use warp::http::StatusCode;
use warp::reply::json;

use common::data::{Pageable, UserRequest};

use crate::{db, Result};
use crate::DBPool;
use crate::error::Error::DBQueryError;

pub async fn health_handler(db_pool: DBPool) -> Result<impl Reply> {
    let db = db::get_conn(&db_pool)
        .await
        .map_err(|e| reject::custom(e))?;
    db.execute("SELECT 1", &[])
        .await
        .map_err(|e| reject::custom(DBQueryError(e)))?;
    Ok(StatusCode::OK)
}

pub async fn get_users(pageable: Pageable, db_pool: DBPool) -> Result<impl Reply> {
    let found_users = db::find_users(&db_pool, pageable).await
        .map_err(|e| reject::custom(e))?;
    Ok(json(
        &found_users
    ))
}

pub async fn create_user(user_request: UserRequest, db_pool: DBPool) -> Result<impl Reply> {
    let created_user = db::create_user(&db_pool, user_request).await
        .map_err(|e| reject::custom(e))?;
    Ok(json(
        &created_user
    ))
}

pub async fn get_tasks(pageable: Pageable, db_pool: DBPool) -> Result<impl Reply> {
    let found_tasks = db::find_tasks(&db_pool, pageable).await
        .map_err(|e| reject::custom(e))?;
    Ok(json(&found_tasks))
}



