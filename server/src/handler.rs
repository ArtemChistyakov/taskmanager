use warp::{reject, Reply};
use warp::http::StatusCode;
use warp::reply::json;

use common::data::{LoginRequest, LoginResponse, Pageable, ProjectRequest, TaskRequest, User, UserDto, UserRequest};

use crate::{auth, db, notification, Result};
use crate::DBPool;
use crate::error::Error::*;

pub async fn health_handler(db_pool: DBPool) -> Result<impl Reply> {
    let db = db::get_conn(&db_pool)
        .await
        .map_err(|e| reject::custom(e))?;
    db.execute("SELECT 1", &[])
        .await
        .map_err(|e| reject::custom(DBQueryError(e)))?;
    Ok(StatusCode::OK)
}

pub async fn login_handler(login_request: LoginRequest, db_pool: DBPool) -> Result<impl Reply> {
    let user = db::find_user_by_email(db_pool, &login_request.email)
        .await
        .map_err(|e| reject::custom(e))?;
    if !user.enabled {
        return Err(reject::custom(UserNotEnabledError));
    }
    let is_right_pwd = bcrypt::verify(login_request.pwd, &user.pwd)
        .map_err(|e| reject::custom(VerifyPasswordError(e)))?;
    if !is_right_pwd {
        return Err(reject::custom(WrongCredentialsError));
    }
    let token = auth::create_token(&user)
        .map_err(|e| reject::custom(e))?;
    Ok(json(&LoginResponse { token }))
}

pub async fn get_users(pageable: Pageable, db_pool: DBPool, user_id: i32) -> Result<impl Reply> {
    let found_users = db::find_users(&db_pool, pageable).await
        .map_err(|e| reject::custom(e))?;
    let dtos = found_users.into_iter()
        .map(Into::into)
        .collect::<Vec<UserDto>>();
    Ok(json(
        &dtos
    ))
}

pub async fn register_user(user_request: UserRequest, db_pool: DBPool) -> Result<impl Reply> {
    let (created_user, verification_token) = db::create_user_and_verification_token(&db_pool, user_request)
        .await
        .map_err(|e| reject::custom(e))?;
    let dto: UserDto = created_user.into();
    notification::send_registration_email(&verification_token)
        .map_err(|e| reject::custom(NotificationError))?;
    Ok(json(
        &dto
    ))
}

pub async fn get_tasks(pageable: Pageable, db_pool: DBPool,
                       user_id: i32) -> Result<impl Reply> {
    let found_tasks = db::find_tasks(&db_pool, pageable, user_id)
        .await
        .map_err(|e| reject::custom(e))?;
    Ok(json(&found_tasks))
}

pub async fn create_task(task_request: TaskRequest, db_pool: DBPool,
                         user_id: i32) -> Result<impl Reply> {
    let created_task = db::create_task(db_pool, task_request, user_id).await
        .map_err(|e| reject::custom(e))?;
    Ok(json(&created_task))
}

pub async fn delete_task(task_id: i32, db_pool: DBPool, user_id: i32) -> Result<impl Reply> {
    db::delete_task(db_pool, task_id, user_id)
        .await
        .map_err(|e| reject::custom(e))?;
    Ok(StatusCode::OK)
}

pub async fn get_projects(pageable: Pageable, db_pool: DBPool,
                          user_id: i32) -> Result<impl Reply> {
    let found_tasks = db::find_projects(&db_pool, pageable, user_id).await
        .map_err(|e| reject::custom(e))?;
    Ok(json(&found_tasks))
}

pub async fn create_project(project_request: ProjectRequest,
                            db_pool: DBPool, user_id: i32) -> Result<impl Reply> {
    let created_project = db::create_project(db_pool, project_request, user_id).await
        .map_err(|e| reject::custom(e))?;
    Ok(json(&created_project))
}

pub async fn delete_project(project_id: i32, db_pool: DBPool, user_id: i32) -> Result<impl Reply> {
    db::delete_project(db_pool, project_id, user_id)
        .await
        .map_err(|e| reject::custom(e))?;
    Ok(StatusCode::OK)
}


