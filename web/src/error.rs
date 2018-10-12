/// Error type that for the Bible.rs application.
#[derive(Fail, Debug)]
pub enum BiblersError {
    #[fail(display = "There a database error")]
    DbError,

    #[fail(display = "There was an error rendering the HTML page.")]
    TemplateError,
}
