#!/bin/bash
set -eu
cd "$(dirname -- "$(type greadlink >/dev/null 2>&1 && greadlink -f -- "$0" || readlink -f -- "$0")")"

# Source .env for SSH_TARGET
[ -e ".env" ] && source ./.env

qmllint qml/**/*.qml

# Make sure the `rpm` is up-to-date.
cargo rpm build

# Query the effective Cargo target directory
TARGET_DIR="$(cargo metadata --format-version=1 | jq -r ".target_directory")"

# Source the versions from the spec file.
VERSION=$(cat "${TARGET_DIR}/armv7-unknown-linux-gnueabihf/release/rpmbuild/SPECS/harbour-whisperfish.spec" | egrep '^Version:' | awk '{ print $2 }')
RELEASE=$(cat "${TARGET_DIR}/armv7-unknown-linux-gnueabihf/release/rpmbuild/SPECS/harbour-whisperfish.spec" | egrep '^Release:' | awk '{ print $2 }')

# Build the RPM filename.
RPM="harbour-whisperfish-$VERSION-$RELEASE.armv7hl.rpm"

echo "Copying RPM file ($RPM)"
rsync "${TARGET_DIR}/armv7-unknown-linux-gnueabihf/release/rpmbuild/RPMS/armv7hl/${RPM}" "nemo@${SSH_TARGET}:/tmp/${RPM}"

echo "Installing RPM file ($RPM)"
ssh "nemo@${SSH_TARGET}" sdk-deploy-rpm "/tmp/${RPM}"

echo "Starting harbour-whisperfish"
# Use -t to force-allocate a terminal, it triggers Qt to log warnings.
ssh -t "nemo@${SSH_TARGET}" "RUST_BACKTRACE=full RUST_LOG=harbour_whisperfish=trace,actix=*,awc=*,actix-web=*,libsignal_service=trace,libsignal_service_actix=trace,debug harbour-whisperfish"
