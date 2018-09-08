#[macro_use]
extern crate clap;
extern crate receptus;

use std::io::{self, Write};

use receptus::establish_connection;
use receptus::reference::Reference;
use receptus::sword_drill::drill;

fn main() -> io::Result<()> {
    let matches = clap_app!(receptuscli =>
        (version: "0.1")
        (author: "Dustin Speckhals <dustin1114@gmail.com>")
        (about: "CLI for looking up Bible verses")
        (@arg REFERENCE: +required "The Bible reference to look up")
    ).get_matches();

    let conn = establish_connection();

    let reference: Reference = matches
        .value_of("REFERENCE")
        .unwrap_or("John 3:16")
        .parse()
        .expect("Invalid reference");
    let result = drill(&reference, &conn);

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
