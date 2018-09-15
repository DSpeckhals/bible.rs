extern crate actix_web;
extern crate diesel;
extern crate diesel_migrations;
extern crate dotenv;
extern crate env_logger;
extern crate handlebars;
extern crate receptus;

use std::env;
use std::error::Error;

use actix_web::{
    fs,
    http::{Method, NormalizePath},
    middleware, server, App,
};
use diesel::r2d2;
use diesel_migrations::run_pending_migrations;
use dotenv::dotenv;
use handlebars::Handlebars;

use receptus::controllers::{api, view};
use receptus::establish_connection;
use receptus::ServerState;
use receptus::SqliteConnectionManager;

fn register_templates() -> Result<Handlebars, Box<Error>> {
    let mut tpl = Handlebars::new();
    tpl.set_strict_mode(true);
    tpl.register_templates_directory(".hbs", "./templates/")?;

    Ok(tpl)
}

#[derive(Default)]
struct StaticFileConfig;

impl fs::StaticFileConfig for StaticFileConfig {
    fn is_use_etag() -> bool {
        true
    }
}

fn main() -> Result<(), Box<Error>> {
    dotenv().ok();

    // Set up logging
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Run DB migrations for a new SQLite database
    run_pending_migrations(&establish_connection()).expect("Error running migrations");

    server::new(move || {
        // Create handlebars registry
        let template = register_templates().unwrap();

        // Create a connection pool
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db = r2d2::Pool::builder()
            .max_size(15)
            .build(SqliteConnectionManager::new(database_url))
            .unwrap();

        // Wire up the application
        App::with_state(ServerState { db, template })
            .handler(
                "/static",
                fs::StaticFiles::with_config("./static", StaticFileConfig).unwrap(),
            ).resource("bible", |r| {
                r.name("bible");
                r.get().with(view::all_books)
            }).resource("bible/{book}", |r| {
                r.name("book");
                r.get().f(view::book)
            }).resource("bible/{reference:.+}", |r| {
                r.name("reference");
                r.get().f(view::index)
            }).resource("api/{reference}.json", |r| r.get().f(api::index))
            .default_resource(|r| r.method(Method::GET).h(NormalizePath::default()))
            .middleware(middleware::Logger::default())
    }).bind("0.0.0.0:8080")
    .unwrap()
    .run();

    Ok(())
}
