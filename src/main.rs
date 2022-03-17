use std::convert::Infallible;

use mobc::{Pool};
use mobc_postgres::PgConnectionManager;
use mobc_postgres::tokio_postgres::NoTls;
use warp::{Filter, Rejection};

mod data;
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
    let sm = warp::path("users")
        .and(warp::query())
        .and(with_db(db_pool.clone()))
        .and_then(handler::get_users)
        .or(warp::path("tasks")
            .and(warp::query())
            .and(with_db(db_pool.clone()))
            .and_then(handler::get_tasks));


    warp::serve(sm).run(([127, 0, 0, 1], 8080)).await;
}

fn with_db(db_pool: DBPool) -> impl Filter<Extract=(DBPool, ), Error=Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}