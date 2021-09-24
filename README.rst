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

Whisperfish is written in Rust, and Rust is a bit of a special guest for SailfishOS.
Luckily, since SailfishOS 3.4, Jolla provides a Rust compiler that runs more or less decently.
Their compiler still has a few issues (which are being discussed and solved),
and in the meanwhile you need a patched version.  This should be resolved by the time
SailfishOS 4.3 tooling and targets are around.
Jolla's Rust compiler works as a cross-compiler, unlike the C and C++ compilers,
which are emulated. This means a bit of preparation is necessary in the *tooling*
as well as in the target.

Make sure you have access to the `sfdk` tool; we will use it for setting up the environment.

1. Make sure you have a tooling and target later than 4.1.0.24.
   Use the SDK Maintenance tool to upgrade if necessary.
2. On versions before 4.1, you need to set up a patched version of Rust.

   First, enable the repository that contains the patched Rust compiler and cargo tooling.
   This repository contains `rubdos.key`, which is used to sign the packages, if you want to check.::

    $ sfdk tools exec SailfishOS-4.1.0.24 ssu ar https://nas.rubdos.be/~rsmet/sailfish-repo/ rubdos
    $ sfdk tools exec SailfishOS-4.1.0.24-armv7hl ssu ar https://nas.rubdos.be/~rsmet/sailfish-repo/ rubdos

   Then, install the tooling::

    $ sfdk tools exec SailfishOS-4.1.0.24 \
        zypper update -y --allow-vendor-change \
          rust cargo \
          rust-std-static-aarch64-unknown-linux-gnu \
          rust-std-static-armv7-unknown-linux-gnueabihf \
          rust-std-static-i686-unknown-linux-gnu

   Then, install the stub compilers (for armv7hl and aarch64)::

    $ sfdk tools exec SailfishOS-4.1.0.24-armv7hl \
        zypper install --repo rubdos -y \
          rust cargo \
          rust-std-static \
          rust-std-static-i686-unknown-linux-gnu

   on i486 use this instead::

    $ zypper install --from rubdos -y rust cargo rust-std-static-i686-unknown-linux-gnu

3. You can now proceed to build as you would with a normal SailfishOS application::

    $ sfdk config --push target SailfishOS-4.1.0.24-armv7hl
    $ sfdk build

Because of a bug in `sb2`, it is currently not possible to (reliably) build using more than a single thread.
This means your compilation is going to take a while, especially the first time.
Get yourself some coffee!

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
