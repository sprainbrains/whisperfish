#!/bin/bash

set -e

echo Building for SailfishOS-$SFOS_VERSION-$TARGET
sb2 -t SailfishOS-$SFOS_VERSION-$TARGET \
    -m sdk-install \
    -R zypper --non-interactive in cmake git make openssl-devel \
        qt5-qtwebsockets qt5-qtwebsockets-devel \


echo Copying source
sudo cp -r . ~nemo/src
sudo chown -R nemo:nemo ~nemo/src

echo Done copying source.
pwd

(
    cd ~nemo/src

    mb2 -t SailfishOS-$SFOS_VERSION-$TARGET build

    echo Done building
)

sudo cp -r /home/nemo/src/RPMS RPMS

# Rust
case "$TARGET" in
    i486 )
        export RUST_TARGET=i586-unknown-linux-gnu ;;
    armv7hl )
        export RUST_TARGET=arm-unknown-linux-gnueabihf ;;
esac

echo Building for Rust target $RUST_TARGET

curl --proto '=https' --tlsv1.2 -sSf -o rustup.sh https://sh.rustup.rs
sb2 -t SailfishOS-$SFOS_VERSION-$TARGET -m sdk-install \
    sh rustup.sh \
        --profile minimal \
        --target $RUST_TARGET \
        -y \

(
    cd ~nemo/src
    sb2 -t SailfishOS-$SFOS_VERSION-$TARGET -m sdk-build \
        cargo build --target=$RUST_TARGET
    echo Done building Rust version
)
