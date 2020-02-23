export RUSTUP_TARGET=i686-unknown-linux-gnu
export RUSTUP_CC=$MERSDK/toolings/SailfishOS-$SFOS_VERSION/opt/cross/bin/i486-meego-linux-gnu-gcc
export RUSTUP_AR=$MERSDK/toolings/SailfishOS-$SFOS_VERSION/opt/cross/bin/i486-meego-linux-gnu-ar

export CC_i686_unknown_linux_gnu="$RUSTUP_CC"
