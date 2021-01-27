#!/bin/bash
set -eu
cd "$(dirname -- "$(type greadlink >/dev/null 2>&1 && greadlink -f -- "$0" || readlink -f -- "$0")")"

# Source .env for SSH_TARGET
[ -e ".env" ] && source ./.env

qmllint qml/**/*.qml

# Make sure the build is up-to-date.
cargo build --target="${TARGET_ARCH}"

# Also make sure translations are up-to-date
./update-translations.sh
for filepath in translations/*.ts;
do
	lrelease -idbased "${filepath}" -qm "${filepath%.*}.qm";
done

# Query the effective Cargo target directory
TARGET_DIR="$(cargo metadata --format-version=1 | jq -r ".target_directory")"

# Copying system files to the target device requires SSH root privileges
#
# To enable this (assuming `sudo` is installed):
#   1. Connect to the target as user `nemo` then:
#     1. Run `sudo nano /etc/ssh/sshd_config`
#     2. Add the line `PermitRootLogin yes` somewhere and exit
#     3. Run `sudo systemctl restart sshd` to apply the changes
#     4. Run `sudo passwd root` and set some temporary root password
#   2. On your computer run `ssh-copy-id root@${SSH_TARGET}` to install your public key on the device
#      (will require the temporary root password)
#   3. You should now be able to use public-key based login for `root@${SSH_TARGET}`
#   4. On the target revert the previous changes:
#     1. Run `sudo nano /etc/ssh/sshd_config` and remove the added `PermitRootLogin` line
#     2. Run `sudo systemctl restart sshd` to apply this change
#     3. Run `sudo passwd -d root` to uninstall the root password

# The list below mirrors the on in `Cargo.toml`'s `package.metadata.rpm.files` and ideally could be deduplicated
echo
echo " • Updating files on target"
rsync -avzP  "${TARGET_DIR}/${TARGET_ARCH}/debug/harbour-whisperfish" "root@${SSH_TARGET}:/usr/bin/harbour-whisperfish"
rsync -avzP  harbour-whisperfish.desktop                              "root@${SSH_TARGET}:/usr/share/applications/harbour-whisperfish.desktop"
rsync -avzP  harbour-whisperfish-message.conf                         "root@${SSH_TARGET}:/usr/share/lipstick/notificationcategories/harbour-whisperfish-message.conf"
rsync -avzP  icons/86x86/harbour-whisperfish.png                      "root@${SSH_TARGET}:/usr/share/icons/hicolor/86x86/apps/harbour-whisperfish.png"
rsync -RavzP qml/ icons/ translations/ --exclude 'translations/*.ts'  "root@${SSH_TARGET}:/usr/share/harbour-whisperfish/"

echo
echo " • Terminating currently running application"
ssh "nemo@${SSH_TARGET}" killall --wait harbour-whisperfish ||:

echo
echo " • Starting harbour-whisperfish"
# Use -t to force-allocate a terminal, it triggers Qt to log warnings.
ssh -t "nemo@${SSH_TARGET}" "RUST_BACKTRACE=full RUST_LOG=harbour_whisperfish=trace,actix=*,awc=*,actix-web=*,libsignal_service=trace,libsignal_service_actix=trace,debug harbour-whisperfish"
