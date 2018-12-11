use failure::Fail;

/// Error type that for the Bible.rs application.
#[derive(Fail, Debug)]
pub enum Error {
    #[fail(
        display = "There was an error with the Actix async arbiter. Cause: {}",
        cause
    )]
    Actix { cause: String },

    #[fail(display = "'{}' was not found.", book)]
    BookNotFound { book: String },

    #[fail(display = "There was a database error.")]
    Db,

    #[fail(display = "'{}' is not a valid Bible reference.", reference)]
    InvalidReference { reference: String },

    #[fail(display = "There was an error rendering the HTML page.")]
    Template,
}
