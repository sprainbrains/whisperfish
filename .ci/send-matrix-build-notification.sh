#!/bin/bash

set -e

CARGO_VERSION="$(grep -m1 -e '^version\s=\s"' Cargo.toml | sed -e 's/.*"\(.*-dev\).*"/\1/')"
echo "Looking for builds for version $CARGO_VERSION"
GIT_REF="$(git rev-parse --short HEAD)"
echo "Filtering on $GIT_REF"
VERSION="$CARGO_VERSION.b$CI_PIPELINE_IID.$GIT_REF"
echo "Complete version would be $VERSION"

BASEVERSION=$(echo $VERSION | sed -e 's/\([0-9]\+\.[0-9]\+\.[0-9]\+\).*/\1/')
echo "Base version is $BASEVERSION"

PACKAGE_ID=$(curl -s "$CI_API_V4_URL/projects/$CI_PROJECT_ID/packages?package_name=harbour-whisperfish" | jq ".[] | select(.version == \"$BASEVERSION\") | .id")
PAGES=$(curl -si -XHEAD "$CI_API_V4_URL/projects/$CI_PROJECT_ID/packages/$PACKAGE_ID/package_files" |grep x-total-pages | sed -e 's/x-total-pages: \([0-9]\+\)/\1/' | tr -d '\n\r')

echo "Checking only page $PAGES"

JQ_FORMAT_LIST="
.[]
    | select(.file_name | test(\"$GIT_REF\"))
    | \"<li><a href=\\\\\\\"$CI_PROJECT_URL/-/package_files/\"+(.id|tostring)+\"/download\\\\\\\">\"+.file_name+\"</a></li>\"
"
DOWNLOAD_LIST=$(curl -s "$CI_API_V4_URL/projects/$CI_PROJECT_ID/packages/$PACKAGE_ID/package_files?page=$PAGES")
echo "Contents of page"
echo $DOWNLOAD_LIST
DOWNLOAD_LIST=$(echo "$DOWNLOAD_LIST" | jq -r "$JQ_FORMAT_LIST")

UNFORMATTED="Builds of $VERSION are ready at https://gitlab.com/rubdos/whisperfish/-/packages"
FORMATTED="🆕 builds of <code>$VERSION</code> are ready at https://gitlab.com/rubdos/whisperfish/-/packages 🥳 <ul>$DOWNLOAD_LIST</ul>"
MSG="{\"msgtype\":\"m.notice\", \"format\": \"org.matrix.custom.html\", \"body\":\"$UNFORMATTED\", \"formatted_body\": \"$FORMATTED\"}"
echo Sending $MSG

curl -XPOST -d "$MSG" "$MATRIX_HOME_SERVER/_matrix/client/r0/rooms/$MATRIX_ROOM/send/m.room.message?access_token=$MATRIX_ACCESS_TOKEN"
