#!/bin/sh

lupdate qml/ -ts translations/*.ts
if git diff --exit-code; then
    echo "No translation update needed";
else
    echo "Committing translation update";
    git config --global user.email "whisperfish@rubdos.be"
    git config --global user.name "Gitlab CI translation file sync"
    git commit translations/ -m "Translation file synchronisation";
    git remote add origin-ssh git@gitlab.com:whisperfish/whisperfish.git
    git push origin-ssh HEAD:master
fi
