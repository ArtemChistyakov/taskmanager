use warp::reject;
use warp::Rejection;
use warp::reply::json;

use crate::DBPool;
use crate::data::{Pageable, UserRequest};
use crate::db;

pub async fn get_users(pageable: Pageable, db_pool: DBPool) -> Result<impl warp::Reply, Rejection> {
    let found_users = db::find_users(&db_pool, pageable).await
        .map_err(|e| reject::custom(e))?;
    Ok(json(
        &found_users
    ))
}

pub async fn create_user(user_request: UserRequest, db_pool: DBPool) -> Result<impl warp::Reply, Rejection> {
    let created_user = db::create_user(&db_pool, user_request).await
        .map_err(|e| reject::custom(e))?;
    Ok(json(
        &created_user
    ))
}

pub async fn get_tasks(pageable: Pageable, db_pool: DBPool) -> Result<impl warp::Reply, Rejection> {
    let found_tasks = db::find_tasks(&db_pool, pageable).await
        .map_err(|e| reject::custom(e))?;
    Ok(json(&found_tasks))
}



