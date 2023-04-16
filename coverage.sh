#!/bin/bash
# shellcheck disable=SC2155
set -e

if which llvm-profdata >/dev/null ; then
	export LLVM_TOOL=$(which llvm-profdata)
else
	echo "Error: llvm-profdata not found"
	exit 1
fi

if ! which grcov >/dev/null ; then
	echo "Error: grcov not found"
	exit 1
fi

export LLVM_PATH=$(dirname "${LLVM_TOOL}")
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="whisperfish-%p-%m.profraw"
export CARGO_TARGET_DIR="coverage"

cargo test

mkdir -p coverage/html

echo "Running grcov, this takes a little while..."

grcov . \
	--binary-path ./coverage/debug/deps/ \
	--branch \
	--ignore-not-existing \
	--ignore '../*' \
	--ignore "/*" \
	--ignore "whisperfish/tests/*" \
	--ignore 'target/*' \
	--ignore 'coverage/*' \
	--llvm-path "${LLVM_PATH}" \
	--excl-start "#\[cfg\(test\)\]" \
	-s . \
	-t html \
	-o coverage/html

find . -name "*.profraw" -delete

if [ -f "coverage/html/index.html" ]; then
	echo "Coverage report available in $(realpath "coverage/html/index.html")"
fi
