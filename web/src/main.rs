#![warn(clippy::all)]

use std::env;
use std::error::Error;
use std::io;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use handlebars::Handlebars;

use db::{build_pool, establish_connection, run_migrations, SqliteConnectionPool, SwordDrill};

use crate::controllers::{api, view};

/// Represents the [server data](actix_web.web.Data.html) for the application.
pub struct ServerData {
    pub db: SqliteConnectionPool,
    pub template: Handlebars<'static>,
}

/// Registers the [Handlebars](handlebars.handlebars.html) templates for the application.
fn register_templates() -> Result<Handlebars<'static>, Box<dyn Error>> {
    let mut tpl = Handlebars::new();
    tpl.set_strict_mode(true);
    tpl.register_templates_directory(".hbs", "./web/templates/")?;

    Ok(tpl)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    // Set up logging
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Get env configuration
    let url = env::var("DATABASE_URL").unwrap_or_else(|_| "/tmp/biblers.db".to_string());

    // Set up sentry
    let _sentry = sentry::init(sentry::ClientOptions::default());

    // Run DB migrations for a new SQLite database
    run_migrations(&mut establish_connection(&url)).expect("Error running migrations");

    let app_data = web::Data::new(ServerData {
        // Create database connection pool
        db: build_pool(&url),
        // Create handlebars registry
        template: register_templates().unwrap(),
    });

    HttpServer::new(move || {
        // Wire up the application
        App::new()
            .wrap(sentry_actix::Sentry::new())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .app_data(app_data.clone())
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
            .default_service(web::route().to(HttpResponse::NotFound))
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
