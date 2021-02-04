#!/bin/bash
#
# Requires watchit (https://github.com/ichthyosaurus/watchit) and pyinotify
# (https://github.com/seb-m/pyinotify).
#
# Environment:
# SSH_TARGET=nemo@phone     how to connect to the target device
# NO_BUILD=1                if building is disabled (building is enabled by default)
#                           This can be specified as environment variable or as
#                           command line argument. If building is disabled, the
#                           executable is expected in build/harbour-whisperfish.
#                           Manually extract it from the latest RPM.
#
# Configuration:
# Change $cSHADOW to a user-writable path on your device. The default is
# /opt/sdk/harbour-whisperfish.
#
# The script will run from Whisperfish's root directory. It watches for changed
# (modified, added, removed) files, then rebuilds and deploys them as needed.

set -eu
cd "$(dirname -- "$(type greadlink >/dev/null 2>&1 && greadlink -f -- "$0" || readlink -f -- "$0")")"

if ! type watchit >/dev/null 2>&1; then
    echo "error: this script requires 'watchit' (https://github.com/ichthyosaurus/watchit)"
fi

[[ -f ".env" ]] && source ./.env  # source .env for SSH_TARGET
cSSH_TARGET="${SSH_TARGET:-"nemo@phone"}"

# base path of files on the target device
cSHADOW="/opt/sdk/harbour-whisperfish"

# Set this to non-null to disable building. Can be specified as environment
# variable or as command line argument (NO_BUILD=1 live-patch.sh)
# The executable is expected in build/harbour-whisperfish. Extract it from the
# latest RPM.
cNO_BUILDING="${NO_BUILD:+no-building}"
[[ "$1" == NO_BUILD* ]] && cNO_BUILDING=no-building

if [[ -z "$cNO_BUILDING" ]]; then
    # query the effective Cargo target directory
    cTARGET_DIR="$(cargo metadata --format-version=1 | jq -r ".target_directory")"
    cTARGET_ARCH='armv7hl'
    cLOCAL_EXE="${cTARGET_DIR}/${cTARGET_ARCH}/debug/harbour-whisperfish"
else
    cLOCAL_EXE=build/harbour-whisperfish  # extract from latest RPM
fi

function restart_app() {
    local cEXEC="usr/bin/harbour-whisperfish"
    local cENVIRONMENT=("RUST_BACKTRACE=full" "RUST_LOG=harbour_whisperfish=trace,actix=*,awc=*,actix-web=*,libsignal_service=trace,libsignal_service_actix=trace,debug")
    # local cENVIRONMENT=("RUST_LOG=debug")

    echo && echo "••••• terminating currently running instance"
    ssh "$cSSH_TARGET" killall --wait "$cSHADOW/$cEXEC" || true
    # Use -tt to force-force-allocate a terminal even though it's in the
    # background. This triggers Qt to print log output.
    ( ssh -tt "$cSSH_TARGET" "${cENVIRONMENT[@]}" "$cSHADOW/$cEXEC" ) &
}

function push() { # 1: source, 2: dest, 3: 'with-mkdir'?
    local src="$1"; local dest="$cSHADOW/$2"; local make_dirs="${3:-no-mkdir}"
    echo " - pushing '$src' to '$dest'"

    if [[ "$make_dirs" == "with-mkdir" ]]; then
        if [[ "$dest" == */ ]]; then
            ssh "$cSSH_TARGET" mkdir -p "$dest" || { error 10 "failed to prepare dir '$dest'"; }
        else
            ssh "$cSSH_TARGET" mkdir -p "$(dirname "$dest")" || { error 10 "failed to prepare dir '$(dirname "$dest")'"; }
        fi
    fi

    rsync -avzP --delete "$src" "$cSSH_TARGET:$dest" || { error 10 "failed to copy files"; }
}

function update_translations() {
    echo && echo "••••• updating translations"
    mkdir -p build/translations
    lupdate-qt5 qml/ -noobsolete -ts translations/*.ts

    for filepath in translations/*.ts; do
        lrelease-qt5 -idbased "${filepath}" -qm "build/${filepath%.*}.qm";
    done
}

function refresh_qml() { # 1: 'with-mkdir'?
    local make_dirs="${1:-no-mkdir}"
    echo && echo "••••• updating QML files on target"
    # qmllint qml/**/*.qml
    update_translations
    push qml                                 "usr/share/harbour-whisperfish/"   "$make_dirs"
    push build/translations                  "usr/share/harbour-whisperfish/" # "$make_dirs"
}

function refresh_files() { # 1: 'with-mkdir'?
    local make_dirs="${1:-no-mkdir}"
    # the list below mirrors the one in Cargo.toml's package.metadata.rpm.files and ideally could be deduplicated
    echo && echo "••••• updating files on target"
    push "$cLOCAL_EXE"                       "usr/bin/harbour-whisperfish" "$make_dirs"
    push harbour-whisperfish.desktop         "usr/share/applications/harbour-whisperfish.desktop" "$make_dirs"
    push harbour-whisperfish-message.conf    "usr/share/lipstick/notificationcategories/harbour-whisperfish-message.conf" "$make_dirs"
    push icons/86x86/harbour-whisperfish.png "usr/share/icons/hicolor/86x86/apps/harbour-whisperfish.png" "$make_dirs"
    push icons                               "usr/share/harbour-whisperfish/" # "$make_dirs"
    refresh_qml "$make_dirs"
}

function main() {
    if [[ -z "$cNO_BUILDING" ]]; then
        # make sure the build is up-to-date
        cargo build --target="${TARGET_ARCH}"
    fi

    refresh_files with-mkdir
    restart_app

    while event="$(watchit . -wsg "$cLOCAL_EXE" '*.qml' '*.qm' '*.ts' '*.js' '*.conf' '*.desktop' '*.png' '*.rs')"; do
        printf "%s\n" "$event"
        if [[ "$event" =~ \.rs$ ]] && [[ -z "$cNO_BUILDING" ]]; then
            cargo build --target="${TARGET_ARCH}"
            refresh_files
        elif [[ "$event" =~ \.qml$ || "$event" =~ \.js$ ]]; then
            refresh_qml
        else
            refresh_files
        fi
        restart_app
    done
}

main
