extern crate actix_web;
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
extern crate handlebars;
extern crate receptus;

use std::env;
use std::error::Error;

use actix_web::{middleware, server, App};
use diesel::r2d2;
use dotenv::dotenv;
use handlebars::Handlebars;

use receptus::controllers::{api, view};
use receptus::ServerState;
use receptus::SqliteConnectionManager;

fn register_templates() -> Result<Handlebars, Box<Error>> {
    let mut tpl = Handlebars::new();
    tpl.register_template_file("base", "./templates/base.hbs")?;
    tpl.register_template_file("view", "./templates/view.hbs")?;
    tpl.register_template_file("not_found", "./templates/not-found.hbs")?;
    Ok(tpl)
}

fn main() -> Result<(), Box<Error>> {
    dotenv().ok();

    // Set up logging
    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

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
            .resource("/api/{reference}.json", |r| r.get().with(api::index))
            .resource("/bible/{reference:.+}", |r| r.get().with(view::index))
            .middleware(middleware::Logger::default())
    }).bind("127.0.0.1:8080")
    .unwrap()
    .run();

    Ok(())
}
