#[macro_use]
extern crate clap;
extern crate db;
extern crate dotenv;

use std::env;
use std::io::{self, Write};

use dotenv::dotenv;

use db::models::Reference;
use db::{establish_connection, sword_drill, VerseFormat};

fn main() -> io::Result<()> {
    let matches = clap_app!(biblerscli =>
        (version: "0.1")
        (author: "Dustin Speckhals <dustin1114@gmail.com>")
        (about: "CLI for looking up Bible verses")
        (@arg REFERENCE: +required "The Bible reference to look up")
    ).get_matches();

    dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = establish_connection(&url);

    let reference: Reference = matches
        .value_of("REFERENCE")
        .unwrap_or("John 3:16")
        .parse()
        .expect("Invalid reference");
    let result = sword_drill::verses(&reference, &VerseFormat::PlainText, &conn);

    match result {
        Ok((book, verses)) => {
            io::stdout().write_fmt(format_args!(
                "{}\n",
                Reference {
                    book: book.name,
                    chapter: reference.chapter,
                    verses: reference.verses
                }
            ))?;
            for v in verses {
                io::stdout().write_fmt(format_args!("{} {}\n", v.verse, v.words))?;
            }
            Ok(())
        }
        Err(e) => io::stderr().write_fmt(format_args!("{:?}", e)),
    }
}
