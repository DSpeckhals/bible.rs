# Bible.rs

![CI](https://github.com/DSpeckhals/bible.rs/workflows/CI/badge.svg)

> A Bible server written in Rust using Actix Web and Diesel

I've often found myself studying the Bible and wanting to simply "find that
one verse." There are numerous Bible apps and websites that are powerful,
full-featured, and helpful; yet I seldom need most of these features.

Bible.rs is completely free to use, but if you'd like to donate, feel free to
with [Paypal](https://paypal.me/DSpeckhals).

## Where Bible.rs Shines
- Simple: just the Bible text.
- Easy-to-navigate: links for everything you might need, referenced by an
obvious URL scheme (e.g. Psalm/119/105).
- Easy-to-find: want to find "that one verse?" Just press "S" on your
keyboard and start typing! You can also search by reference, including
a wide range of book abbreviations (e.g. "1tim" for "First Timothy").
- Speed: all-around quick page loads.

## The Logo
<img src="https://raw.githubusercontent.com/DSpeckhals/bible.rs/master/web/dist/img/bible.rs.svg?sanitize=true" alt="Bible.rs logo" height="100" width="100">

- Fire and hammer: the power and efficacy of the Word of God illustrated in
[Jeremiah 23:29](https://bible.rs/Jeremiah/23#v29).
- Gear: the practicality and industry of the Rust programming language.

## The Technology
I built Bible.rs to be a simple, fast, and usable window into the King James
version of the Bible. The website is lightweight, with JavaScript only used
for the search box (though it is also fully usable with JavaScript disabled).
The HTML is meant to be semantic and thus easily machine parsable.

The "brains" of Bible.rs are written in [Rust]("https://www.rust-lang.org/"),
a programming language I had toyed around with, but never did anything
substantial with. That's why this is called "Bible.*rs*": "rs" is the file
extension for Rust source files. The data is stored in a single [SQLite](https://www.sqlite.org/index.html)
database, with the actual Bible text sourced from [Robert Rouse](https://github.com/robertrouse/KJV-bible-database-with-metadata-MetaV-).

I also don't keep any of your personal information. There are no tracking
cookies or third-party analytic services involved in Bible.rs.

## Ideas for the Future
- Add more metadata (book author, year written, etc.).
- Improve Schema.org metadata.
- Add languages other than English.
- When and if SQLite is ever a bottleneck, switch to a client-server database.
- Make the searching "smart."

## Contributing
- Have any ideas? [File an issue](https://github.com/DSpeckhals/bible.rs/issues/new).
- Want to venture into the code? Clone the repository from
[Github](https://github.com/DSpeckhals/bible.rs) and create a pull request.

## Docker
1. Pull the Git repository, including submodules

    `git pull https://github.com/DSpeckhals/bible.rs.git`

If you've already pulled the repo but not the migrations submodule, run `git submodule update --init --recursive`

2. To run the Docker container for Bible.rs

    `docker build -t biblers . && docker run -p 8080:8080 --rm -it biblers`

3. Navigate to `localhost:8080`
