===============================================================================
Whisperfish
===============================================================================

Whisperfish is a native `Signal <https://www.whispersystems.org/>`_ client for
`Sailfish OS <https://sailfishos.org/>`_. The user interface is heavily based on
the jolla-messages application written by Jolla Ltd.

It is currently *in non working state*, except where noted otherwise.  Join our
development channel on Matrix
(`#whisperfish:rubdos.be <https://matrix.to/#/#whisperfish:rubdos.be>`_) or
Freenode (#whisperfish) to get in touch.

-------------------------------------------------------------------------------
Project Status
-------------------------------------------------------------------------------

This project was based of a now outdated Go-based SailfishOS client for Signal.
This version is a port that uses `libsignal-c-protocol
<https://github.com/signalapp/libsignal-protocol-c>`_ instead.
This means we aim for better maintainability.
It also means the whole SailfishOS app needs to be rewritten, and you may want
to make a back-up of your current files. Specifically:

- `.local/share/harbour-whisperfish/` contains all your data.
- `.config/harbour-whisperfish/` contains the apps configuration.

-------------------------------------------------------------------------------
Features
-------------------------------------------------------------------------------

Currently, only this is re-implemented:

- [x] Open and read existing encrypted database.

We are currently aiming for 0.5 compatibility, which means on 0.6 release we
have these features:

- [x] Registration
- [x] Contact Discovery
- [x] Direct messages
- [x] Group messages
- [x] Storing conversations
- [x] Photo attachments
- [x] Encrypted identity and session store
- [x] Encrypted message store
- [x] Advanced user settings
- [ ] Multi-Device support (links with Signal Desktop)
- [ ] Encrypted local attachment store
- [ ] Archiving conversations

Please search the `issue tracker <https://gitlab.com/rubdos/whisperfish/-/issues>`_
before filing a regression issue from 0.5.
Feel free to post feature requests for features that were *not* available in 0.6,
however!

-------------------------------------------------------------------------------
Nightly builds
-------------------------------------------------------------------------------

The most recent builds can be found here:

- armv7hl: https://gitlab.com/rubdos/whisperfish/-/jobs/artifacts/master/browse?job=build:armv7hl
- i486: https://gitlab.com/rubdos/whisperfish/-/jobs/artifacts/master/browse?job=build:i486


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


1. Clone the repository::

    $ git clone https://gitlab.com/rubdos/whisperfish

2. Whisperfish is built from the `Platform SDK <https://sailfishos.org/wiki/Platform_SDK>`, outside of the classic ``sb2`` and ``mb2`` environments.
   Refer to the Platform SDK page for installation instructions.
   After that, `install the relevant targets <https://sailfishos.org/wiki/Platform_SDK_Target_Installation>` for the device you are building for,
   e.g.::

    sdk-assistant create SailfishOS-latest-armv7hl http://releases.sailfishos.org/sdk/targets/Sailfish_OS-latest-Sailfish_SDK_Target-armv7hl.tar.7z

4. Still in the SDK chroot use ``sdk-manage`` to install the Sqlite-sqlcipher build dependency::

   sdk-manage develpkg install SailfishOS-latest-armv7hl sqlcipher-devel

5. Make a copy of ``dotenv.example`` to ``.env``, adapt it to your configuration and source it.

6. Since the libsignal-c library is built using `cmake <https://cmake.org/>`_,
   we need cmake *in the build environment*.
   You can install it from within the SDK.
   We also need `openssl-devel` for the cryptographic provider.
   If you prefer to install it over the command line, `ssh` into your build system and use `zypper`::

    $ ssh -p 2222 -i ~/SailfishOS/vmshare/ssh/private_keys/engine/mersdk mersdk@localhost
    $ sudo zypper -n install cmake make git openssl-devel qt5-qtwebsockets-devel

7. For building on the host, ie. running ``cargo test`` or whatever you may desire, the Debian
   requirements are in ``Dockerfile.builder``, reproduced here::

           $ sudo apt-get install -y \
                   build-essential
                   gcc-i686-linux-gnu g++-i686-linux-gnu binutils-i686-linux-gnu \
                   gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf binutils-arm-linux-gnueabihf \
                   gcc-aarch64-linux-gnu g++-aarch64-linux-gnu binutils-aarch64-linux-gnu \
                   curl \
                   qtbase5-dev \
                   qt5-qmake \
                   qtdeclarative5-dev \
                   qttools5-dev-tools qtchooser qt5-default \
                   desktop-file-utils \
                   rpm \
                   cmake \
                   libsqlcipher-dev

   You will also be needing some Rust things::

           $ rustup toolchain install nightly
           $ rustup target add armv7-unknown-linux-gnueabihf
           $ cargo install --git https://github.com/RustRPM/cargo-rpm --branch develop

8. From here on, you can use cargo to build the project;
   make sure to have the correct targets installed (rustup target) and a C compiler set::

    $ cargo build --release --target=armv7-unknown-linux-gnueabihf

   The ``harbour-whisperfish`` executable resides in ``target/[target]/release``.
   You can also use ``cargo rpm`` to build an RPM package,
   note that you need ``rpmtools`` installed on the host system::

    $ cargo install cargo-rpm
    $ cargo rpm build

   The generated RPM can be found in ``target/[target]/release/rpmbuild/RPMS/armv7hl/``.

-------------------------------------------------------------------------------
Testing on the device
-------------------------------------------------------------------------------

The ``run.sh`` script will will source the ``.env`` file and run the build on your device.

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
i18n Translations (help wanted)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Whisperfish supports i18n translations and uses Text ID Based Translations. See
`here <http://doc.qt.io/qt-5/linguist-id-based-i18n.html>`_ for more info. To
translate the application strings in your language run (for example German)::

    $ ssh -p 2222 -i ~/SailfishOS/vmshare/ssh/private_keys/engine/mersdk mersdk@localhost
    $ cd $GOPATH/src/github.com/aebruno/whisperfish
    $ sb2 lupdate qml/ -ts qml/i18n/whisperfish_de.ts
    [edit whisperfish_de.ts]
    $ sb2 lrelease -idbased qml/i18n/whisperfish_de.ts -qm qml/i18n/whisperfish_de.qm

-------------------------------------------------------------------------------
License
-------------------------------------------------------------------------------

Copyright (C) 2016-2018 Andrew E. Bruno
Copyright (C) 2019-2020 Ruben De Smet and contributors

Whisperfish is free software: you can redistribute it and/or modify it under the
terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
this program. If not, see <http://www.gnu.org/licenses/>.
