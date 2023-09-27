#!/bin/bash

set -e

echo "Building for $SFOS_VERSION"
# The MB2 image comes with a default user.
# We need to copy the source over, because of that.

git clone . ~/whisperfish-build
pushd ~/whisperfish-build

git status

cd "shareplugin_v$SHAREPLUGIN_VERSION"

# -f to ignore non-existent files
rm -f RPMS/*.rpm

mb2 -t SailfishOS-$SFOS_VERSION-$MER_ARCH build \
    --enable-debug \
    --no-check

[ "$(ls -A RPMS/*.rpm)" ] || exit 1

# Copy everything useful back
popd
mkdir -p RPMS target
sudo cp -ar ~/whisperfish-build/shareplugin_v$SHAREPLUGIN_VERSION/RPMS/* RPMS/

# Only upload on tags or main
if [ -n "$CI_COMMIT_TAG" ] || [[ "$CI_COMMIT_BRANCH" == "main" ]]; then
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
