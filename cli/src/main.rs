use clap::Parser;

use std::env;
use std::io::{self, Write};

use dotenv::dotenv;

use db::models::Reference;
use db::{establish_connection, SwordDrill, SwordDrillable, VerseFormat};

#[derive(Parser, Debug)]
#[clap(
    version = "0.1",
    author = "Dustin Speckhals <dustin1114@gmail.com>",
    about = "CLI for looking up Bible verses"
)]
struct Opts {
    #[clap(default_value = "John 3:16")]
    reference: Reference,
}

fn main() -> io::Result<()> {
    let opts: Opts = Opts::parse();
    let reference = opts.reference;

    dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = establish_connection(&url);

    let result = SwordDrill::verses(&reference, VerseFormat::PlainText, &conn);

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
                io::stdout().write_fmt(format_args!("{}\t{}\n", v.verse, v.words))?;
            }
            Ok(())
        }
        Err(e) => io::stderr().write_fmt(format_args!("{:?}", e)),
    }
}
