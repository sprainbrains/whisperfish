export RUSTUP_TARGET=armv7-unknown-linux-gnueabi
export RUSTUP_CC=$MERSDK/toolings/SailfishOS-$SFOS_VERSION/opt/cross/bin/armv7hl-meego-linux-gnueabi-gcc
export RUSTUP_AR=$MERSDK/toolings/SailfishOS-$SFOS_VERSION/opt/cross/bin/armv7hl-meego-linux-gnueabi-ar

export CC_armv7_unknown_linux_gnueabi="$RUSTUP_CC"

# XXX: potentially also add +neon
export RUSTFLAGS="$RUSTFLAGS -C target-feature=+v7"
