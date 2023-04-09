#!/bin/sh

. .ci/load-ssh-key

git config --global user.email "whisperfish@rubdos.be"
git config --global user.name "Gitlab CI cargo update"

WHITELIST="libsignal-service libsignal-service-actix libsignal-protocol"

git branch -D ci-dependency-bump || echo "Creating a clean ci-dependency-bump branch"
git checkout -b ci-dependency-bump

for crate in $WHITELIST; do
    echo "Attempting update of $crate"
    cargo update -p $crate

    if git diff --exit-code; then
        echo "No update needed";
    else
        echo "Committing crate update";
        git commit Cargo.lock -m "Update $crate dependency";
        git remote add origin-ssh git@gitlab.com:whisperfish/whisperfish.git
    fi
done

if git diff --exit-code $CI_DEFAULT_BRANCH; then
    echo "No push needed";
else
    git push \
        -o merge_request.create \
        -o merge_request.title="Automatic dependency update" \
        -o merge_request.merge_when_pipeline_succeeds \
        --force-with-lease \
        origin-ssh ci-dependency-bump
fi
