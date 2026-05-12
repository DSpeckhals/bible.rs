#![warn(clippy::all)]

use std::env;
use std::error::Error;
use std::io;

use actix_files::NamedFile;
use actix_web::dev::Service;
use actix_web::http::header;
use actix_web::middleware::DefaultHeaders;
use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use dotenv::dotenv;
use futures_util::future::{Either, FutureExt};
use handlebars::Handlebars;
use log::info;

use db::{
    SqliteConnectionPool, SwordDrill, build_pool, establish_connection, prefetch_books,
    run_migrations,
};

use crate::controllers::{api, view};

/// Represents the [server data](actix_web.web.Data.html) for the application.
pub struct ServerData {
    pub books: Vec<db::models::Book>,
    pub db: SqliteConnectionPool,
    pub template: Handlebars<'static>,
}

/// Registers the [Handlebars](handlebars.handlebars.html) templates for the application.
fn register_templates() -> Result<Handlebars<'static>, Box<dyn Error>> {
    let mut tpl = Handlebars::new();
    tpl.set_strict_mode(true);
    let mut opts = handlebars::DirectorySourceOptions::default();
    opts.tpl_extension = ".hbs".to_string();
    tpl.register_templates_directory("./web/templates/", opts)?;

    Ok(tpl)
}

async fn robots_txt() -> std::io::Result<NamedFile> {
    NamedFile::open("./web/dist/robots.txt")
}

async fn sitemap_xml() -> std::io::Result<NamedFile> {
    NamedFile::open("./web/dist/sitemap.xml")
}

const SERVICE_WORKER_JS: &[u8] = include_bytes!("../dist/js/sw.js");

async fn service_worker() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(("Service-Worker-Allowed", "/"))
        .insert_header((header::CONTENT_TYPE, "application/javascript"))
        .body(SERVICE_WORKER_JS)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    // Default log level is `info`; honors `RUST_LOG` if set.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Get env configuration
    let url = env::var("DATABASE_URL").unwrap_or_else(|_| "/tmp/biblers.db".to_string());
    info!("Database: {url}");

    // Run migrations and preload book data on a single non-pooled connection.
    let mut startup_conn = establish_connection(&url);
    run_migrations(&mut startup_conn).expect("Error running migrations");
    let books = prefetch_books(&mut startup_conn).expect("Error preloading books");
    drop(startup_conn);

    let app_data = web::Data::new(ServerData {
        db: build_pool(&url),
        books,
        template: register_templates().expect("Error registering templates"),
    });

    HttpServer::new(move || {
        // Wire up the application
        App::new()
            // Redirect www.bible.rs to the apex (Fly does the HTTP->HTTPS redirect)
            .wrap_fn(|req, srv| {
                let host_is_www = req
                    .headers()
                    .get(header::HOST)
                    .and_then(|h| h.to_str().ok())
                    .map(|h| {
                        h.split(':')
                            .next()
                            .unwrap_or(h)
                            .eq_ignore_ascii_case("www.bible.rs")
                    })
                    .unwrap_or(false);
                if host_is_www {
                    let path_and_query = req
                        .uri()
                        .path_and_query()
                        .map(|pq| pq.as_str())
                        .unwrap_or("/");
                    let location = format!("https://bible.rs{path_and_query}");
                    let resp = req.into_response(
                        HttpResponse::MovedPermanently()
                            .insert_header((header::LOCATION, location))
                            .finish(),
                    );
                    Either::Left(async move { Ok(resp) }.boxed_local())
                } else {
                    Either::Right(srv.call(req))
                }
            })
            .wrap(
                DefaultHeaders::new()
                    .add((
                        "Strict-Transport-Security",
                        "max-age=63072000; includeSubDomains; preload",
                    ))
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
                    .add((
                        "Permissions-Policy",
                        "geolocation=(), microphone=(), camera=()",
                    ))
                    .add((
                        "Content-Security-Policy",
                        "default-src 'self'; \
                         script-src 'self' 'unsafe-inline'; \
                         style-src 'self'; \
                         img-src 'self' data:; \
                         font-src 'self'; \
                         object-src 'none'; \
                         base-uri 'self'; \
                         form-action 'self'; \
                         frame-ancestors 'self'; \
                         manifest-src 'self'",
                    )),
            )
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .app_data(app_data.clone())
            // sw.js needs Service-Worker-Allowed: / so its scope can be the site root.
            // Must be registered before the /static Files service.
            .service(web::resource("/static/js/sw.js").route(web::get().to(service_worker)))
            // Font filenames are content-stable, so mark them immutable for
            // a year. Must be registered before the catch-all /static Files.
            .service(
                web::scope("/static/fonts")
                    .wrap(
                        DefaultHeaders::new()
                            .add(("Cache-Control", "public, max-age=31536000, immutable")),
                    )
                    .service(actix_files::Files::new("", "./web/dist/fonts").use_etag(true)),
            )
            .service(actix_files::Files::new("/static", "./web/dist").use_etag(true))
            .service(web::resource("about").to(view::about))
            .service(
                web::resource("/")
                    .name("bible")
                    .route(web::get().to(view::all_books)),
            )
            .service(web::resource("search").route(web::get().to(view::search::<SwordDrill>)))
            // robots.txt and sitemap.xml must be registered before the {book} catch-all,
            // which would otherwise match them as a book name.
            .service(web::resource("/robots.txt").route(web::get().to(robots_txt)))
            .service(web::resource("/sitemap.xml").route(web::get().to(sitemap_xml)))
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
    .bind(env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()))?
    .run()
    .await
}

mod controllers;
mod error;
mod macros;
mod responder;
#[cfg(test)]
mod test;
