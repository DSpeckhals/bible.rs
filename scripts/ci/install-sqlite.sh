# Remove existing sqlite3
sudo apt remove sqlite3 libsqlite3-dev

SQLITE_BUILD_DIR=$TRAVIS_BUILD_DIR/.sqlite-build

mkdir $SQLITE_BUILD_DIR
cd $SQLITE_BUILD_DIR
curl -sL https://www.sqlite.org/2019/sqlite-autoconf-3280000.tar.gz | tar -xzC $SQLITE_BUILD_DIR
$SQLITE_BUILD_DIR/sqlite-autoconf-3280000/configure --enable-fts5 --disable-fts3 --disable-fts4
make
sudo make install
sudo ldconfig
cd $TRAVIS_BUILD_DIR
