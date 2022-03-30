use std::convert::Infallible;

use mobc::Pool;
use mobc_postgres::PgConnectionManager;
use mobc_postgres::tokio_postgres::NoTls;
use warp::{Filter, Rejection};

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
    let users = warp::path("users");
    let tasks = warp::path("tasks");
    let health_route = warp::path!("health")
        .and(with_db(db_pool.clone()))
        .and_then(handler::health_handler);

    let user_routes = users
        .and(warp::get())
        .and(warp::query())
        .and(with_db(db_pool.clone()))
        .and_then(handler::get_users)
        .or(
            users.and(warp::post())
                .and(warp::body::json())
                .and(with_db(db_pool.clone()))
                .and_then(handler::create_user)
        );
    let task_routes = tasks
        .and(warp::query())
        .and(with_db(db_pool.clone()))
        .and_then(handler::get_tasks);


    let routes = health_route
        .or(user_routes)
        .or(task_routes)
        .with(warp::cors().allow_any_origin())
        .recover(error::handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

fn with_db(db_pool: DBPool) -> impl Filter<Extract=(DBPool, ), Error=Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}