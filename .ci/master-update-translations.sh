#!/bin/sh

. .ci/load-ssh-key
lupdate qml/ -ts translations/*.ts
if git diff --exit-code; then
    echo "No translation update needed";
else
    echo "Committing translation update";
    git commit translations/ -m "Translation file synchronisation";
    git push origin master
fi
