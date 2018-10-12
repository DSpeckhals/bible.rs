extern crate actix;
extern crate actix_web;
extern crate db;
extern crate dotenv;
extern crate env_logger;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate handlebars;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

use std::env;
use std::error::Error;

use actix::{Addr, SyncArbiter, System};
use actix_web::{
    fs,
    http::{Method, NormalizePath},
    middleware, server, App,
};
use dotenv::dotenv;
use handlebars::Handlebars;

use db::{build_pool, establish_connection, run_migrations, SqliteConnectionPool};

use actors::{DbExecutor};
use controllers::{api, view};

/// Represents the [server state](actix_web.ServerState.html) for the application.
pub struct ServerState {
    pub db1: Addr<DbExecutor>,
    pub db: SqliteConnectionPool,
    pub template: Handlebars,
}

/// Registers the [Handlebars](handlebars.handlebars.html) templates for the application.
fn register_templates() -> Result<Handlebars, Box<Error>> {
    let mut tpl = Handlebars::new();
    tpl.set_strict_mode(true);
    tpl.register_templates_directory(".hbs", "./web/templates/")?;

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

    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Run DB migrations for a new SQLite database
    run_migrations(&establish_connection(&url)).expect("Error running migrations");

    let sys = System::new("biblers-db-arbitors");

    let url_clone = url.clone();
    let addr = SyncArbiter::start(10, move || {
        DbExecutor(build_pool(&url_clone))
    });

    server::new(move || {
        // Create handlebars registry
        let template = register_templates().unwrap();

        // Create a connection pool
        let db = build_pool(&url);

        // Wire up the application
        App::with_state(ServerState { db1: addr.clone(), db, template })
            .handler(
                "/static",
                fs::StaticFiles::with_config("./web/dist", StaticFileConfig).unwrap(),
            ).resource("about", |r| r.get().with(view::about))
            .resource("/", |r| {
                r.name("bible");
                r.get().with(view::all_books)
            }).resource("search", |r| r.get().f(view::search))
            .resource("{book}", |r| {
                r.name("book");
                r.get().f(view::book)
            }).resource("{reference:.+\\d}", |r| {
                r.name("reference");
                r.get().f(view::reference)
            }).resource("api/search", |r| r.get().f(api::search))
            .resource("api/{reference}.json", |r| r.get().f(api::reference))
            .default_resource(|r| r.method(Method::GET).h(NormalizePath::default()))
            .middleware(middleware::Logger::default())
    }).bind("0.0.0.0:8080")
    .unwrap()
    .run();

    let _ = sys.run();

    Ok(())
}

mod actors;
mod error;
mod controllers;
