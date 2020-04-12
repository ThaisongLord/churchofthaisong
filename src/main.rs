use mobc::{Connection, Pool};
use mobc_postgres::{tokio_postgres, PgConnectionManager};
use serde_derive::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_postgres::NoTls;
use warp::{Filter, Rejection};

mod db;
mod error;
mod handler;

type Result<T> = std::result::Result<T, Rejection>;
type DBCon = Connection<PgConnectionManager<NoTls>>;
type DBPool = Pool<PgConnectionManager<NoTls>>;

#[derive(Deserialize, Serialize)]
pub struct Todo {
    pub id: Option<u64>,
    pub name: String,
    pub ordering: usize,
    pub checked: bool,
}

#[tokio::main]
async fn main() {
    let db_pool = db::create_pool().expect("database pool can be created.");

    db::init_db(&db_pool)
        .await
        .expect("database can be initialized");

    let health_route = warp::path!("health")
        .and(with_db(db_pool.clone()))
        .and_then(handler::health_handler);

    let todo = warp::path!("todo");
    let todo_routes = todo
        .and(with_db(db_pool.clone()))
        .and_then(handler::list_todos_handler)
        .or(todo
            .and(warp::post())
            .and(warp::body::json())
            .and(with_db(db_pool.clone()))
            .and_then(handler::create_todo_handler))
        .or(todo
            .and(warp::path::param())
            .and(warp::put())
            .and(warp::body::json())
            .and(with_db(db_pool.clone()))
            .and_then(handler::update_todo_handler))
        .or(todo
            .and(warp::path::param())
            .and(warp::delete())
            .and(with_db(db_pool.clone()))
            .and_then(handler::delete_todo_handler));

    let routes = health_route
        .or(todo_routes)
        .recover(error::handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_db(db_pool: DBPool) -> impl Filter<Extract = (DBPool,), Error = Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}