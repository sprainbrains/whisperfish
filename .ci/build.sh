#!/bin/bash

set -e

echo Building for SailfishOS-$SFOS_VERSION-$TARGET
sb2 -t SailfishOS-$SFOS_VERSION-$TARGET \
    -m sdk-install \
    -R zypper --non-interactive in cmake git make
echo Copying source
sudo cp -r . ~nemo/src
sudo chown -R nemo:nemo ~nemo/src
cd ~nemo/src

echo Done copying source.
pwd

mb2 -t SailfishOS-$SFOS_VERSION-$TARGET build

echo Done building
pwd
ls RPMS
