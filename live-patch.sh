#!/bin/bash
#
# Configuration:
# Change $cSHADOW to a user-writable path on your device. The default is
# /opt/sdk/harbour-whisperfish.
#
# The script will run from Whisperfish's root directory. It can watch for changed
# (modified, added, removed) files, build and deploy Whisperfish, and restart
# the app on the target device.
#
# By default, it will deploy and restart the app once, then quit.
#
# Arguments:
# -h, --help                just a hint to this comment
# -b, --build               enable building, cf. BUILDING below
# -w, --watcher             enable watching the file system, cf. WATCHER below
# -B, --no-build            disable building, overrides environment (default)
# -W, --no-watcher          disable watching the file system, overrides env. (default)
#
# Environment:
# SSH_TARGET=nemo@phone     How to connect to the target device
#                           (defaults to 'nemo@phone')
# BUILDING=foo              Set to anything to enable building (building is disabled
#                           by default). This can be specified as environment variable
#                           or as command line argument (-b). If building is disabled, the
#                           executable is expected in build/harbour-whisperfish.
#                           Manually extract it from the latest RPM.
# WATCHER=foo               Set to anything to enable the file system watcher
#                           (disabled by default).
# TARGET_ARCH=armv7hl       Change to build for a different architecture
# LUPDATE_TOOL=lupdate      Change this if your system packages e.g. lupdate-qt5
# LRELEASE_TOOL=lrelease    Change this if your system packages e.g. lrelease-qt5
#
# Note: watching (-w) requires watchit (https://github.com/ichthyosaurus/watchit)
# and pyinotify (https://github.com/seb-m/pyinotify).
#

set -eu
cd "$(dirname -- "$(type greadlink >/dev/null 2>&1 && greadlink -f -- "$0" || readlink -f -- "$0")")"

if printf -- "%s\n" "$@" | grep -qoExe "--help|-h"; then
    printf "usage: %s\n" "$0"
    echo "Please refer to the comments at the top of the script."
    exit 0
fi

# Enable the watcher if --watcher is given or WATCHER is set in the environment.
cWATCHER="${WATCHER:+yes}"
cWATCHER="${cWATCHER:-no}"
if printf -- "%s\n" "$@" | grep -qoExe "--watcher|-w"; then
    cWATCHER='yes'
    if ! type watchit >/dev/null 2>&1; then
        echo "error: this script requires 'watchit' (https://github.com/ichthyosaurus/watchit)"
    fi
elif printf -- "%s\n" "$@" | grep -qoExe "--no-watcher|-W"; then
    cWATCHER='no'
fi

[[ -f ".env" ]] && source ./.env  # source .env for SSH_TARGET
cSSH_TARGET="${SSH_TARGET:-"nemo@phone"}"

# base path of files on the target device
cSHADOW="/opt/sdk/harbour-whisperfish"

# Enable building if --build is given or BUILDING is set in the environment.
# If building is disabled, the executable is expected in build/harbour-whisperfish.
# Extract it from the latest RPM.
cBUILDING="${BUILDING:+yes}"
cBUILDING="${cBUILDING:-no}"
if printf -- "%s\n" "$@" | grep -qoExe "--build|-b"; then
    cBUILDING='yes'
elif printf -- "%s\n" "$@" | grep -qoExe "--no-build|-B"; then
    cBUILDING='no'
fi

cLUPDATE_TOOL="${LUPDATE_TOOL:-lupdate}"
cLRELEASE_TOOL="${LRELEASE_TOOL:-lrelease}"

if [[ "$cBUILDING" == 'yes' ]]; then
    # query the effective Cargo target directory
    cTARGET_DIR="$(cargo metadata --format-version=1 | jq -r ".target_directory")"
    cTARGET_ARCH="${TARGET_ARCH:-"armv7hl"}"
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
    "$cLUPDATE_TOOL" qml/ -noobsolete -ts translations/*.ts

    for filepath in translations/*.ts; do
        "$cLRELEASE_TOOL" -idbased "${filepath}" -qm "build/${filepath%.*}.qm";
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
    refresh_qml # "$make_dirs" (shared path already prepared for icons)
}

function main() {
    if [[ "$cBUILDING" == 'yes' ]]; then
        # make sure the build is up-to-date
        cargo build --target="${TARGET_ARCH}"
    fi

    # We omit mkdir in the loop because preparing directories once should be
    # safe enough (tm), and it is very slow.
    refresh_files with-mkdir
    restart_app

    if [[ "$cWATCHER" == 'no' ]]; then
        return $?
    fi

    while event="$(watchit . -wsg "$cLOCAL_EXE" '*.qml' '*.qm' '*.js' '*.conf' '*.desktop' '*.png' '*.rs')"; do
        printf "%s\n" "$event"
        if [[ "$event" =~ \.rs$ ]] && [[ "$cBUILDING" == 'yes' ]]; then
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

[[ "$cWATCHER" == 'yes' ]] && echo "note: watcher enabled"
[[ "$cBUILDING" == 'yes' ]] && echo "note: building enabled"

main
