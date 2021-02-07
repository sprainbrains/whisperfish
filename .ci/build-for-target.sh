#!/bin/bash

mkdir -p $CARGO_HOME
cp .ci/cargo.toml $CARGO_HOME/config

echo "Building for $SFOS_VERSION"
echo "Configuring cargo-rpm (cfr https://gitlab.com/rubdos/whisperfish/-/issues/24)"

if [ -z "$CI_COMMIT_TAG" ]; then
    CARGO_VERSION="$(grep -m1 -e '^version\s=\s"' Cargo.toml | sed -e 's/.*"\(.*\)"/\1/')"
    GIT_REV="$(git rev-list --count HEAD)"
    GIT_REF="$(git rev-parse --short HEAD)"
    VERSION="$CARGO_VERSION.r$GIT_REV.$GIT_REF"
else
    # Strip leading v in v0.6.0- ...
    VERSION=$(echo "$CI_COMMIT_TAG" | sed -e 's/^v//g')
fi

# Configure Cargo.toml
sed -ie "s/armv7hl/$MER_ARCH/" Cargo.toml
sed -ie "s/^version\s\?=\s\?\".*\"/version = \"$VERSION\"/" Cargo.toml
cat Cargo.toml

# Set env
export MER_TARGET="SailfishOS-$SFOS_VERSION"
export RUSTFLAGS="-C link-args=-Wl,-lcrypto,-rpath-link,$MERSDK/targets/$MER_TARGET-$MER_ARCH/usr/lib64/,-rpath-link,$MERSDK/targets/$MER_TARGET-$MER_ARCH/usr/lib/,-rpath-link,$MERSDK/targets/$MER_TARGET-$MER_ARCH/lib/,-rpath-link,$MERSDK/targets/$MER_TARGET-$MER_ARCH/lib64/"

rustc --version
cargo --version

cargo-rpm --help
cargo rpm build --verbose --target $RUST_ARCH
