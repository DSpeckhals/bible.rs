#![warn(clippy::all)]

use std::env;
use std::error::Error;
use std::io;

use actix_web::{http::ContentEncoding, middleware, web, App, HttpServer};
use dotenv::dotenv;
use handlebars::Handlebars;

use db::{build_pool, establish_connection, run_migrations, SqliteConnectionPool, SwordDrill};
use sentry_actix::SentryMiddleware;

use crate::controllers::{api, view};

/// Represents the [server data](actix_web.web.Data.html) for the application.
pub struct ServerData {
    pub db: SqliteConnectionPool,
    pub template: Handlebars,
}

/// Registers the [Handlebars](handlebars.handlebars.html) templates for the application.
fn register_templates() -> Result<Handlebars, Box<dyn Error>> {
    let mut tpl = Handlebars::new();
    tpl.set_strict_mode(true);
    tpl.register_templates_directory(".hbs", "./web/templates/")?;

    Ok(tpl)
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    // Set up logging
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Get env configuration
    let sentry_dsn = env::var("SENTRY_DSN").ok();
    let url = env::var("DATABASE_URL").unwrap_or_else(|_| "/tmp/biblers.db".to_string());

    // Set up sentry
    let capture_errors = sentry_dsn.is_some();
    let _guard = sentry::init(sentry_dsn);
    if capture_errors {
        sentry::integrations::panic::register_panic_handler();
    }

    // Run DB migrations for a new SQLite database
    run_migrations(&establish_connection(&url)).expect("Error running migrations");

    let pool = build_pool(&url);

    HttpServer::new(move || {
        // Create handlebars registry
        let template = register_templates().unwrap();

        // Wire up the application
        App::new()
            .wrap(middleware::Compress::new(ContentEncoding::Gzip))
            .wrap(
                SentryMiddleware::new()
                    .emit_header(true)
                    .capture_server_errors(capture_errors),
            )
            .wrap(middleware::Logger::default())
            .data(ServerData {
                db: pool.clone(),
                template,
            })
            .service(actix_files::Files::new("/static", "./web/dist").use_etag(true))
            .service(web::resource("about").to(view::about))
            .service(
                web::resource("/")
                    .name("bible")
                    .route(web::get().to(view::all_books::<SwordDrill>)),
            )
            .service(web::resource("search").route(web::get().to(view::search::<SwordDrill>)))
            .service(
                web::resource("{book}")
                    .name("book")
                    .route(web::get().to(view::book::<SwordDrill>)),
            )
            .service(
                web::resource("{reference:.+\\d}")
                    .name("reference")
                    .route(web::get().to(view::reference::<SwordDrill>)),
            )
            .service(web::resource("api/search").route(web::get().to(api::search::<SwordDrill>)))
            .service(
                web::resource("api/{reference}.json")
                    .route(web::get().to(api::reference::<SwordDrill>)),
            )
            .default_service(web::route().to(web::HttpResponse::NotFound))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

mod controllers;
mod error;
mod macros;
mod responder;
#[cfg(test)]
mod test;
