use warp::Rejection;
use warp::reject;
use warp::reply::json;

use crate::{DBPool, error};
use crate::data::Pageable;
use crate::db;

pub async fn get_users(pageable: Pageable, db_pool: DBPool) -> Result<impl warp::Reply, Rejection> {
    let found_users = db::find_users(&db_pool, &pageable).await
        .map_err(|e| reject::custom(e))?;
    Ok(json(
        &found_users
    ))
}

pub async fn get_tasks(pageable: Pageable, db_pool: DBPool) -> Result<impl warp::Reply, Rejection> {
    let found_tasks = db::find_tasks(&db_pool, &pageable).await
        .map_err(|e| reject::custom(e))?;
    Ok(json(&found_tasks))
}

