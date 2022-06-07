#!/bin/bash

set -e

echo "Building for $SFOS_VERSION"

sudo zypper install -y \
    sqlcipher-devel \
    openssl-devel \
    zlib-devel \

# Tooling-side dependencies used in build.rs
sdk-manage tooling maintain SailfishOS-$SFOS_VERSION \
    zypper install -y \
        sqlcipher-devel \
        openssl-devel \
        zlib-devel \

# For i486, we lie.
# https://gitlab.com/whisperfish/whisperfish/-/issues/24

if [ -z "$CI_COMMIT_TAG" ]; then
    CARGO_VERSION="$(grep -m1 -e '^version\s=\s"' Cargo.toml | sed -e 's/.*"\(.*-dev\).*"/\1/')"
    GIT_REF="$(git rev-parse --short HEAD)"
    VERSION="$CARGO_VERSION.b$CI_PIPELINE_IID.$GIT_REF"
else
    # Strip leading v in v0.6.0- ...
    VERSION=$(echo "$CI_COMMIT_TAG" | sed -e 's/^v//g')
fi

# The MB2 image comes with a default user.
# We need to copy the source over, because of that.

git clone . ~/whisperfish-build
pushd ~/whisperfish-build

# We also need to move the cache, and afterwards move it back.
if [ -e "$CI_PROJECT_DIR/cargo" ]; then
    sudo mv $CI_PROJECT_DIR/cargo ~/cargo
    sudo chown -R $USER:$USER ~/cargo
fi

git status

# -f to ignore non-existent files
rm -f RPMS/*.rpm

mb2 -t SailfishOS-$TARGET_VERSION-$MER_ARCH build \
    --enable-debug \
    --no-check \
    -- \
    --define "dist $DIST" \
    --define "cargo_version $VERSION"\
    --with lto \
    --with sccache \


[ "$(ls -A RPMS/*.rpm)" ] || exit 1

# Copy everything useful back
popd
mkdir -p RPMS target
sudo cp -ar ~/whisperfish-build/RPMS/* RPMS/
sudo cp -ar ~/whisperfish-build/target/* target/

sudo mv ~/cargo $CI_PROJECT_DIR/cargo

# Only upload on tags or master
if [ -n "$CI_COMMIT_TAG" ] || [[ "$CI_COMMIT_BRANCH" == "master" ]]; then
    for RPM_PATH in RPMS/*.rpm; do
        echo Found RPM: $RPM_PATH
        RPM_PATH="${RPM_PATH[0]}"
        RPM=$(basename $RPM_PATH)

        URL="$CI_API_V4_URL/projects/$CI_PROJECT_ID/packages/generic/harbour-whisperfish/$VERSION/$RPM"
        echo Posting to $URL

        # Upload to Gitlab
        curl --header "PRIVATE-TOKEN: $PRIVATE_TOKEN" \
             --upload-file "$RPM_PATH" \
             $URL
    done
fi
