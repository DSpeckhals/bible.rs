use std::env;
use std::error::Error;

use actix_web::{
    actix::*,
    fs,
    http::{ContentEncoding, Method, NormalizePath},
    middleware, server, App,
};
use dotenv::dotenv;
use handlebars::Handlebars;

use db::{build_pool, establish_connection, run_migrations};

use crate::actors::DbExecutor;
use crate::controllers::{api, view};

/// Represents the [server state](actix_web.ServerState.html) for the application.
pub struct ServerState {
    pub db: Addr<DbExecutor>,
    pub template: Handlebars,
}

/// Registers the [Handlebars](handlebars.handlebars.html) templates for the application.
fn register_templates() -> Result<Handlebars, Box<dyn Error>> {
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

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Set up logging
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Run DB migrations for a new SQLite database
    run_migrations(&establish_connection(&url)).expect("Error running migrations");

    let sys = System::new("biblers");
    let addr = SyncArbiter::start(num_cpus::get(), move || DbExecutor(build_pool(&url)));

    server::new(move || {
        // Create handlebars registry
        let template = register_templates().unwrap();

        // Wire up the application
        App::with_state(ServerState {
            db: addr.clone(),
            template,
        })
        .default_encoding(ContentEncoding::Gzip)
        .handler(
            "/static",
            fs::StaticFiles::with_config("./web/dist", StaticFileConfig).unwrap(),
        )
        .resource("about", |r| r.get().with(view::about))
        .resource("/", |r| {
            r.name("bible");
            r.get().with(view::all_books)
        })
        .resource("search", |r| r.get().f(view::search))
        .resource("{book}", |r| {
            r.name("book");
            r.get().f(view::book)
        })
        .resource("{reference:.+\\d}", |r| {
            r.name("reference");
            r.get().f(view::reference)
        })
        .resource("api/search", |r| r.get().f(api::search))
        .resource("api/{reference}.json", |r| r.get().f(api::reference))
        .default_resource(|r| r.method(Method::GET).h(NormalizePath::default()))
        .middleware(middleware::Logger::default())
    })
    .bind("0.0.0.0:8080")
    .unwrap()
    .start();

    let _ = sys.run();

    Ok(())
}

mod actors;
mod controllers;
mod error;
