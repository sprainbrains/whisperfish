#!/bin/bash

set -e

CARGO_VERSION="$(grep -m1 -e '^version\s=\s"' Cargo.toml | sed -e 's/.*"\(.*\)"/\1/')"
GIT_REF="$(git rev-parse --short HEAD)"
VERSION="$CARGO_VERSION.b$CI_PIPELINE_IID.$GIT_REF"

PACKAGE_ID=$(curl -s "$CI_API_V4_URL/projects/$CI_PROJECT_ID/packages?package_name=harbour-whisperfish" | jq ".[] | select(.version == \"$CARGO_VERSION\") | .id")
PAGES=$(curl -si -XHEAD "$CI_API_V4_URL/projects/$CI_PROJECT_ID/packages/$PACKAGE_ID/package_files" |grep x-total-pages | sed -e 's/x-total-pages: //')

echo "Checking only page $PAGES"

JQ_FORMAT_LIST="
.[]
    | select(.file_name | test(\"$GIT_REF\"))
    | \"<li><a href=\\\"$CI_PROJECT_URL/-/package_files/\"+(.id|tostring)+\"/download\\\">\"+.file_name+\"</a></li>\"
"
DOWNLOAD_LIST=$(curl -s "$CI_API_V4_URL/projects/$CI_PROJECT_ID/packages/$PACKAGE_ID/package_files?page=$PAGES")
echo "Contents of page ("$CI_API_V4_URL/projects/$CI_PROJECT_ID/packages/$PACKAGE_ID/package_files?page=$PAGES") $PAGES"
echo $DOWNLOAD_LIST
DOWNLOAD_LIST=$(echo "$DOWNLOAD_LIST" | jq -r "$JQ_FORMAT_LIST")

FORMATTED="🆕 New builds🧱 ($VERSION) are ready at https://gitlab.com/rubdos/whisperfish/-/packages 🥳 <ul>$DOWNLOAD_LIST</ul>"
MSG="{\"msgtype\":\"m.text\", \"format\": \"org.matrix.custom.html\", \"body\":\"New builds are ready at https://gitlab.com/rubdos/whisperfish/-/packages\", \"formatted_body\": \"$FORMATTED\"}"
echo Sending $MSG

curl -XPOST -d "$MSG" "$MATRIX_HOME_SERVER/_matrix/client/r0/rooms/$MATRIX_ROOM/send/m.room.message?access_token=$MATRIX_ACCESS_TOKEN"
