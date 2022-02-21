#!/bin/bash -e

cd "$(dirname "$0")"
SCRIPT_DIR=$(pwd)
SQLCIPHER_VERSION="3.4.2"

# Excerpt from https://github.com/rusqlite/rusqlite/pull/860
# Download and generate sqlcipher amalgamation
mkdir -p "$SCRIPT_DIR/sqlcipher.src"
[ -e "v${SQLCIPHER_VERSION}.tar.gz" ] || curl -sfL -O "https://github.com/sqlcipher/sqlcipher/archive/v${SQLCIPHER_VERSION}.tar.gz"
sha256sum -c v${SQLCIPHER_VERSION}.tar.gz.sha256sum

[ -e "sqlite3.c" ] && exit 0

tar xzf "v${SQLCIPHER_VERSION}.tar.gz" --strip-components=1 -C "$SCRIPT_DIR/sqlcipher.src"
cd "$SCRIPT_DIR/sqlcipher.src"
cp /usr/share/automake-1.*/config.guess config.guess
./configure --with-crypto-lib=none
make sqlite3.c
cp -a sqlite3.c sqlite3.h sqlite3ext.h LICENSE -t "$SCRIPT_DIR"
cd "$SCRIPT_DIR"
rm -rf sqlcipher.src
