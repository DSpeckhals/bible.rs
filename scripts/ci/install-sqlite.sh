#!/bin/bash

set -e

SQLITE_VERSION=3400100
SQLITE_BUILD_DIR=$HOME/.sqlite-build

mkdir $SQLITE_BUILD_DIR
cd $SQLITE_BUILD_DIR
curl -sL "https://sqlite.org/2022/sqlite-autoconf-$SQLITE_VERSION.tar.gz" | tar -xzC $SQLITE_BUILD_DIR
$SQLITE_BUILD_DIR/sqlite-autoconf-$SQLITE_VERSION/configure --enable-fts5 --disable-fts3 --disable-fts4
make
sudo make install
sudo ldconfig
