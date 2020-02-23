#!/bin/bash

set -e

echo Building for SailfishOS-$SFOS_VERSION-$TARGET

export MERSDK=/srv/mer
export MER_TARGET="SailfishOS-$SFOS_VERSION"
export RUSTFLAGS="-C link-args=-Wl,-rpath-link,$MERSDK/targets/$MER_TARGET-$TARGET/usr/lib/,-rpath-link,$MERSDK/targets/$MER_TARGET-$TARGET/lib/"

# Rust
curl --proto '=https' --tlsv1.2 -sSf -o ~nemo/rustup.sh https://sh.rustup.rs
sh ~nemo/rustup.sh \
    --profile minimal \
    -y \

source $HOME/.cargo/env

source ./$TARGET.sh
rustup target add $RUSTUP_TARGET

cat <<EOF > ~/.cargo/config
[target.$RUSTUP_TARGET]
linker = "$RUSTUP_CC"
ar = "$RUSTUP_AR"
EOF

cargo build

sudo cp -r /home/nemo/src/RPMS RPMS
