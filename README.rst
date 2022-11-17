===============================================================================
Whisperfish
===============================================================================

Whisperfish is a native `Signal <https://www.whispersystems.org/>`_ client for
`Sailfish OS <https://sailfishos.org/>`_. The user interface is heavily based on
the jolla-messages application written by Jolla Ltd.

It is currently mostly working state.  Join our development channel on Matrix
(`#whisperfish:rubdos.be <https://matrix.to/#/#whisperfish:rubdos.be>`_) or
Libera.Chat (#whisperfish) to get in touch, and check
`our wiki <https://gitlab.com/whisperfish/whisperfish/-/wikis/home>`_ to see whether
Whisperfish would work for you.

To install, you have two options:

- Releases `from OpenRepos <https://openrepos.net/content/rubdos/whisperfish>`_
- "Nightly" builds from Git development commits.
  The most recent builds can be found in `the Gitlab Package Registry <https://gitlab.com/whisperfish/whisperfish/-/packages>`_.

There's no particular reason to install from Git directly.  We push regular updates
to OpenRepos, when they make sense.

Please mind that Whisperfish in still in *alpha condition*, which means that
certain things do not work, other things make the application crash, and I've
heard reports that alpha software can be a cause for dogs eating homework.
You've been warned.
On the other hand, we have many people happily using Whisperfish as daily driver,
and we make up for lacking features in our community support in the aforementioned
Matrix and IRC room.
Please come say hello! We don't bite (we may byte), and we don't eat homework.

-------------------------------------------------------------------------------
Project Status
-------------------------------------------------------------------------------

This project started from a now outdated Go-based SailfishOS client for Signal.
This version, 0.6 and onwards, is a complete rewrite, and uses `libsignal-client
<https://github.com/signalapp/libsignal-client>`_ instead.
This means we aim for better maintainability.
It also means the whole SailfishOS app had to be rewritten, and you may want
to make a back-up of your current files if you still come from 0.5. Specifically:

- `.local/share/harbour-whisperfish/` contains all your data.
- `.config/harbour-whisperfish/` contains the apps configuration.

-------------------------------------------------------------------------------
Features
-------------------------------------------------------------------------------

- [x] Registration
- [ ] Contact Discovery
- [x] Direct messages
- [x] Group messages, but not yet `GroupV2 <https://gitlab.com/groups/whisperfish/-/epics/1>`_
- [x] Storing conversations
- [x] Photo attachments
- [x] Encrypted identity and session store
- [x] Encrypted message store
- [x] Advanced user settings
- [ ] Multi-Device support (links with Signal Desktop)
- [ ] Encrypted local attachment store
- [ ] Archiving conversations

Please search the `issue tracker <https://gitlab.com/whisperfish/whisperfish/-/issues>`_
before filing any bug report or feature request.
Please upvote issues that are important to you.  We use the vote counter for
determining a feature's priority.

-------------------------------------------------------------------------------
Performance Tips
-------------------------------------------------------------------------------

Whisperfish connects to Signal using Websockets. For a better user experience
try adjusting the power settings on your Jolla to disable late suspend [1].
This should keep the network interfaces up and allow Whisperfish to maintain
websocket connections even when the device is in "sleep". This could
potentially impact your battery life depending on your usage. Otherwise
every time your device goes into deep sleep, the Websocket connection is broken
and you may not receive messages until the next time the OS wakes up and
Whisperfish reconnects.

To disable late suspend and enable "early suspend" run::

    $ mcetool --set-suspend-policy=early    

See here for more information.

1. https://together.jolla.com/question/55056/dynamic-pm-in-jolla/
2. http://talk.maemo.org/showpost.php?p=1401956&postcount=29
3. https://sailfishos.org/wiki/Sailfish_OS_Cheat_Sheet#Blocking_Device_Suspend

-------------------------------------------------------------------------------
Building from source
-------------------------------------------------------------------------------

Whisperfish is written in Rust (and QML), and Rust is a bit of a special entity
in Sailfish OS. Luckily, Jolla has provided a more or less decent Rust compiler
since Sailfish OS 3.4, but it has a few issues, at least up to and including
Sailfish OS 4.4. The issues are already fixed and upstreamed, but as they are
not yet released (this _should_ happen with SFOS 4.5),
compiling Whisperfish requires a patched version of Rust.

Unlike the C and C++ compilers, which are emulated, the Rust compiler in SFDK
works as a cross-compiler. This means a bit of preparation is necessary in the
_tooling_ as well as in the target.

The installation instructions below are for 4.4.0.58, but they work with 4.3.0.12
and 4.1.0.24 as well. Make sure you have both the tooling and the target(s)
installed. Use the SDK Maintenance tool to install or upgrade if necessary.

Only the Docker build engine supports Rust compiling.
VirtualBox build engine will not work.

If you are running Sailfish SDK 3.9.6 or older and/or your build target
is 4.4.0.58 or older, you need to set up a patched version of Rust.

1. Make sure you have access to the `sfdk` tool; we will use it for setting
   up the environment.

   Clone Whisperfish to a subdirectory of your SFDK project root,
   just like any other Sailfish OS project.

2. Enable the repository that contains the patched Rust compiler and cargo tooling.
   This repository contains `rubdos.key`, which is used to sign the packages, if you want to check.::

    $ sfdk tools exec SailfishOS-4.4.0.58 \
        ssu ar https://nas.rubdos.be/~rsmet/sailfish-repo/ rubdos
    $ sfdk tools exec SailfishOS-4.4.0.58-aarch64 \
        ssu ar https://nas.rubdos.be/~rsmet/sailfish-repo/ rubdos
    $ sfdk tools exec SailfishOS-4.4.0.58-armv7hl \
        ssu ar https://nas.rubdos.be/~rsmet/sailfish-repo/ rubdos
    $ sfdk tools exec SailfishOS-4.4.0.58-i486 \
        ssu ar https://nas.rubdos.be/~rsmet/sailfish-repo/ rubdos

3. Install the tooling and development packages::

    $ sfdk tools exec SailfishOS-4.4.0.58 \
      zypper install --oldpackage -y \
        rust=1.52.1+git3-1 \
        cargo=1.52.1+git3-1 \
        rust-std-static-aarch64-unknown-linux-gnu=1.52.1+git3-1 \
        rust-std-static-armv7-unknown-linux-gnueabihf=1.52.1+git3-1 \
        rust-std-static-i686-unknown-linux-gnu=1.52.1+git3-1
    $ sfdk engine exec \
      sudo zypper install -y openssl-devel sqlcipher-devel

4. Install the stub compilers.

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

5. You can now proceed to build as you would with a normal SailfishOS application::

   Choose your architecture::

    $ sfdk config --push target SailfishOS-4.4.0.58-aarch64 # or
    $ sfdk config --push target SailfishOS-4.4.0.58-armv7hl # or
    $ sfdk config --push target SailfishOS-4.4.0.58-i486

   Then start the build::

    $ sfdk build

   Or in a single command. e.g. for aarch64::

    $ sfdk -c target=SailfishOS-4.4.0.58-aarch64 build

6. If you want to also build the sharing plugin for SFOS 4.3+, use this command::

    $ sfdk -c target=SailfishOS-4.4.0.58-aarch64 build -- --with shareplugin_v2

   For Sailfish 4.2 and older, use `shareplugin_v1` instead.

Because of a bug in `sb2`, it is currently not possible to (reliably) build
Whisperfish (or any other Rust project) using more than a single thread.
This means your compilation is going to take a while, especially the first time.
Get yourself some coffee!

If you get errors (command not found or status 126) at linking stage, make sure
that you are not using `~/.cargo/config` to override linkers or compilers.

-------------------------------------------------------------------------------
Development environment tips, tricks and hacks
-------------------------------------------------------------------------------

See doc: `Cool hacks for development <doc/dev-env-hacks.rst>`_

-------------------------------------------------------------------------------
i18n Translations (help wanted)
-------------------------------------------------------------------------------

Whisperfish supports i18n translations and uses Text ID Based Translations. See
`here <http://doc.qt.io/qt-5/linguist-id-based-i18n.html>`_ for more info. For
an easy way to help translating, you can join on
`Weblate <https://hosted.weblate.org/engage/whisperfish/>`_.

-------------------------------------------------------------------------------
License
-------------------------------------------------------------------------------

Before Whisperfish 0.6.0-alpha.1, "the Rust port", Whisperfish was licensed under
the GNU General Public License.  Since Whisperfish 0.6.0-alpha.1, Whisperfish links
to AGPLv3 code, and as such is a combined work as meant under clause 13 of the GPLv3.

The original GPLv3 licensed code that is still contained in this repository,
still falls under GPLv3, as per the copyright of Andrew E. Bruno.
This is the original license statement:

Copyright (C) 2016-2018 Andrew E. Bruno

Whisperfish is free software: you can redistribute it and/or modify it under the
terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
this program. If not, see <http://www.gnu.org/licenses/>.


This is the license statement since 2019, since Whisperfish 0.6.0-alpha.1.

Copyright (C) 2019-2020 Ruben De Smet, Markus Törnqvist

Whisperfish is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

Whisperfish is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
