# Whisperfish

Whisperfish is a native [Signal](https://www.whispersystems.org/) client
for [Sailfish OS](https://sailfishos.org/). The user interface is
heavily based on the jolla-messages application written by Jolla Ltd.

The current branch is an adaptation for Aurora OS

## Project Status

This project started from a now outdated Go-based SailfishOS client for
Signal. This version, 0.6 and onwards, is a complete rewrite, and uses
[libsignal-client](https://github.com/signalapp/libsignal-client)
instead. This means we aim for better maintainability. It also means the
whole SailfishOS app had to be rewritten, and you may want to make a
back-up of your current files if you still come from 0.5. Specifically:

- `.local/share/harbour-whisperfish` contains all your data.
- `.config/harbour-whisperfish` contains the apps configuration.

In current releases the paths have changed:

- `.local/share/be.rubdos/harbour-whisperfish`
- `.config/be.rubdos/harbour-whisperfish`

## Features

- [x] Registration
- [ ] Contact Discovery
- [x] Direct messages
- [x] Group messages
- [x] Sealed sending
- [x] Storing conversations
- [x] Sending attachments
- [x] Taking a photo as an attachment
- [ ] Taking a video as an attachment
- [x] Encrypted identity and session store
- [x] Encrypted message store
- [x] Advanced user settings
- [x] Multi-Device support (links with Signal Desktop)
- [ ] Encrypted local attachment store
- [x] Archiving conversations
- [x] Muting conversations

Please search the
[issue tracker](https://gitlab.com/whisperfish/whisperfish/-/issues) before
filing any bug report or feature request. Please upvote issues that are
important to you. We use the vote counter for determining a feature's
priority. (Only original repo)


## Building from source

Whisperfish is written in Rust (and QML), and Rust is a bit of a special
entity in Sailfish OS. Luckily, Jolla has provided a more or less decent
Rust compiler since Sailfish OS 3.4, but it had some issues, which were
[https://github.com/sailfishos/rust/pull/14](fixed) only in Sailfish OS
4.5. Using the corresponding Sailfish SDK 3.10.4 is *highly* recommended.

**Note:** Only the Docker build engine supports Rust compiling. VirtualBox build engine will not work.

Building Whisperfish for yourself is, despite its size and it being a Rust
project, very simple! Just install Sailfish SDK 3.10.4 or newer, download the
source and compile it! (Older SDK versions and build targets work too, but
they need a good amount of setup. Please see
[the previous version](https://gitlab.com/whisperfish/whisperfish/-/blob/1af345a5deb7900e9dd540aacb57b3ba50fb6cd8/README.rst)
of the readme for instructions.)

When you have the Aurora PSDK up and running, and the Whisperfish sources fetched,
it compiles just like any other native Aurora OS/Sailfish OS application.
For Aurora OS you need to set up a patched version of Rust.

1.Make sure you have access to the `sfdk` tool; we will use it for setting
   up the environment.

   Clone Whisperfish to a subdirectory of your SFDK project root,
   just like any other Sailfish OS project.

2.Enable the repository that contains the patched Rust compiler and cargo tooling.
   This repository contains `rubdos.key`, which is used to sign the packages, if you want to check.::

    $ sfdk tools exec SailfishOS-4.4.0.58 \
        ssu ar https://nas.rubdos.be/~rsmet/sailfish-repo/ rubdos
    $ sfdk tools exec SailfishOS-4.4.0.58-aarch64 \
        ssu ar https://nas.rubdos.be/~rsmet/sailfish-repo/ rubdos
    $ sfdk tools exec SailfishOS-4.4.0.58-armv7hl \
        ssu ar https://nas.rubdos.be/~rsmet/sailfish-repo/ rubdos
    $ sfdk tools exec SailfishOS-4.4.0.58-i486 \
        ssu ar https://nas.rubdos.be/~rsmet/sailfish-repo/ rubdos

3.Install the tooling and development packages::

    $ sfdk tools exec SailfishOS-4.4.0.58 \
      zypper install --oldpackage -y \
        rust=1.52.1+git3-1 \
        cargo=1.52.1+git3-1 \
        rust-std-static-aarch64-unknown-linux-gnu=1.52.1+git3-1 \
        rust-std-static-armv7-unknown-linux-gnueabihf=1.52.1+git3-1 \
        rust-std-static-i686-unknown-linux-gnu=1.52.1+git3-1
    $ sfdk engine exec \
      sudo zypper install -y openssl-devel sqlcipher-devel

4.Install the stub compilers.

   For aarch64::

    $ sfdk tools exec SailfishOS-4.4.0.58-armv7hl \
      zypper install --oldpackage --repo rubdos -y \
        rust=1.52.1+git3-1 \
        cargo=1.52.1+git3-1 \
        rust-std-static-aarch64-unknown-linux-gnu=1.0+git3-1 \
        rust-std-static-i686-unknown-linux-gnu=1.0+git3-1

   For armv7hl::

    $ sfdk tools exec SailfishOS-4.4.0.58-armv7hl \
      zypper install --oldpackage --repo rubdos -y \
        rust=1.52.1+git3-1 \
        cargo=1.52.1+git3-1 \
        ust-std-static-armv7-unknown-linux-gnueabihf=1.0+git3-1 \
        rust-std-static-i686-unknown-linux-gnu=1.0+git3-1

   For i486::

    $ sfdk tools exec SailfishOS-4.4.0.58-i486 \
      zypper install --oldpackage --repo rubdos -y \
        rust=1.52.1+git3-1 \
        cargo=1.52.1+git3-1 \
        rust-std-static-i686-unknown-linux-gnu=1.52.1+git3-1

   If there are errors with the commands, you can try adding `--allow-vendor-change`
   to explicitly tell it's ok to use "third-party" packages, and `--force` to
   re-download and re-install packages.

5.You can now proceed to build as you would with a normal SailfishOS application::

   Choose your architecture::

    $ sfdk config --push target SailfishOS-4.4.0.58-aarch64 # or
    $ sfdk config --push target SailfishOS-4.4.0.58-armv7hl # or
    $ sfdk config --push target SailfishOS-4.4.0.58-i486

   Then start the build::

    $ sfdk build

   Or in a single command. e.g. for aarch64::

    $ sfdk -c target=SailfishOS-4.4.0.58-aarch64 build

6.If you want to also build the sharing plugin for SFOS 4.3+, use this command::

    $ sfdk -c target=SailfishOS-4.4.0.58-aarch64 build -- --with shareplugin_v2

   For Sailfish 4.2 and older, use `shareplugin_v1` instead.


## Development environment tips, tricks and hacks

See doc: [Cool hacks for development](doc/dev-env-hacks.md)

## i18n Translations (help wanted)

Whisperfish supports i18n translations and uses
[Text ID Based Translations](http://doc.qt.io/qt-5/linguist-id-based-i18n.html).
For an easy way to help translating, you can join on
[Weblate](https://hosted.weblate.org/engage/whisperfish/).

## License

Before Whisperfish 0.6.0-alpha.1, "the Rust port", Whisperfish was
licensed under the GNU General Public License. Since Whisperfish
0.6.0-alpha.1, Whisperfish links to AGPLv3 code, and as such is a
combined work as meant under clause 13 of the GPLv3.

The original GPLv3 licensed code that is still contained in this
repository, still falls under GPLv3, as per the copyright of Andrew E.
Bruno. This is the original license statement:

Copyright (C) 2016-2018 Andrew E. Bruno

Whisperfish is free software: you can redistribute it and/or modify it
under the terms of the GNU General Public License as published by the
Free Software Foundation, either version 3 of the License, or (at your
option) any later version.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
Public License for more details.

You should have received a copy of the GNU General Public License along
with this program. If not, see \<<http://www.gnu.org/licenses/>\>.

This is the license statement since 2019, since Whisperfish
0.6.0-alpha.1.

Copyright (C) 2019-2020 Ruben De Smet, Markus Törnqvist

Whisperfish is free software: you can redistribute it and/or modify it
under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or (at
your option) any later version.

Whisperfish is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero
General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program. If not, see
\<<https://www.gnu.org/licenses/>\>.
