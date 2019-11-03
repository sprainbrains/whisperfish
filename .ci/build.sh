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

# Rust
curl --proto '=https' --tlsv1.2 -sSf -o ~nemo/rustup.sh https://sh.rustup.rs
sb2 -t SailfishOS-$SFOS_VERSION-$TARGET -m sdk-install \
    sh ~nemo/rustup.sh \
        --profile minimal \
        -y \

source $HOME/.cargo/env

(
    cd ~nemo/src
    sb2 -t SailfishOS-$SFOS_VERSION-$TARGET -m sdk-build \
        cargo build
    echo Done building Rust version
)

sudo cp -r /home/nemo/src/RPMS RPMS
