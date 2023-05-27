#!/bin/sh

lupdate qml/ -ts translations/*.ts

if [ "$CI" = "true" ]; then
    if git diff --exit-code; then
        echo "No translation update needed";
    else
        echo "Committing translation update";
        git config --global user.email "whisperfish@rubdos.be"
        git config --global user.name "Gitlab CI translation file sync"
        git commit translations/ -m "Translation file synchronisation";
        git remote add origin-ssh git@gitlab.com:whisperfish/whisperfish.git

        . .ci/load-ssh-key
        git push origin-ssh HEAD:master
    fi
fi
