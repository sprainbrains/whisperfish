#!/bin/bash

set -e

echo Building for SailfishOS-$SFOS_VERSION-$TARGET
sb2 -t SailfishOS-$SFOS_VERSION-$TARGET \
    -m sdk-install \
    -R zypper --non-interactive in cmake git make
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
