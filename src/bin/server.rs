extern crate actix_web;
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
extern crate receptus;

use std::env;

use actix_web::{middleware, server, App};
use diesel::r2d2;
use dotenv::dotenv;

use receptus::controllers::api;
use receptus::ServerState;
use receptus::SqliteConnectionManager;

fn main() {
    dotenv().ok();

    // Set up logging
    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    // Create a connection pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = r2d2::Pool::builder()
        .max_size(15)
        .build(SqliteConnectionManager::new(database_url))
        .unwrap();

    server::new(move || {
        App::with_state(ServerState { db: pool.clone() })
            .resource("/ref/{reference}.json", |r| r.get().with(api::index))
            .middleware(middleware::Logger::default())
    }).bind("127.0.0.1:8080")
    .unwrap()
    .run();
}
