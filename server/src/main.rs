use std::convert::Infallible;

use mobc::Pool;
use mobc_postgres::PgConnectionManager;
use mobc_postgres::tokio_postgres::NoTls;
use warp::{Filter, Rejection};

use crate::auth::Role;

mod auth;
mod db;
mod error;
mod handler;

type Result<T> = std::result::Result<T, Rejection>;
type DBPool = Pool<PgConnectionManager<NoTls>>;

#[tokio::main]
async fn main() {
    let db_pool = db::create_pool().unwrap();
    db::db_init(&db_pool)
        .await
        .unwrap();
    let login = warp::path("login");
    let users = warp::path("users");
    let projects = warp::path("projects");
    let tasks = warp::path("tasks");

    let health_route = warp::path!("health")
        .and(with_db(db_pool.clone()))
        .and_then(handler::health_handler);

    let login_route = login
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db_pool.clone()))
        .and_then(handler::login_handler);

    let user_routes = users
        .and(warp::get())
        .and(warp::query())
        .and(with_db(db_pool.clone()))
        .and(auth::with_auth(vec!(Role::Admin)))
        .and_then(handler::get_users)
        .or(
            users.and(warp::post())
                .and(warp::body::json())
                .and(with_db(db_pool.clone()))
                .and_then(handler::create_user)
        );
    let project_routes = projects
        .and(warp::get())
        .and(warp::query())
        .and(with_db(db_pool.clone()))
        .and(auth::with_auth(vec!(Role::User, Role::Admin)))
        .and_then(handler::get_projects)
        .or(projects
            .and(warp::post())
            .and(warp::body::json())
            .and(with_db(db_pool.clone()))
            .and(auth::with_auth(vec!(Role::User, Role::Admin)))
            .and_then(handler::create_project))
        .or(projects
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_db(db_pool.clone()))
            .and(auth::with_auth(vec!(Role::User,Role::Admin)))
            .and_then(handler::delete_project));

    let task_routes = tasks
        .and(warp::get())
        .and(warp::query())
        .and(with_db(db_pool.clone()))
        .and(auth::with_auth(vec!(Role::User, Role::Admin)))
        .and_then(handler::get_tasks)
        .or(
            tasks
                .and(warp::post())
                .and(warp::body::json())
                .and(with_db(db_pool.clone()))
                .and(auth::with_auth(vec!(Role::User, Role::Admin)))
                .and_then(handler::create_task)
        );


    let routes = health_route
        .or(login_route)
        .or(user_routes)
        .or(task_routes)
        .or(project_routes)
        .with(warp::cors().allow_any_origin())
        .recover(error::handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

fn with_db(db_pool: DBPool) -> impl Filter<Extract=(DBPool, ), Error=Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

