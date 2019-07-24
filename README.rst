===============================================================================
Whisperfish
===============================================================================

**WARNING - DO NOT USE THIS CLIENT**

Unfortunately I am no longer able to maintain this project and can not
recommend anyone continue using Whisperfish. The code base is far from complete
and does not implement the latest features of the Signal protocol. It's just a
matter of time before this just completely stops working.  Rather than wait
until that time, I'm archiving this project and recommend any remaining users
switch to another client. This repository is archived and here for historical
purposes only.

Thanks to all who contributed and provided valuable feedback.

Cheers!

**WARNING - DO NOT USE THIS CLIENT**

-------------------------------------------------------------------------------
Project Status
-------------------------------------------------------------------------------

This project is no longer maintained. Do not use this client.

The following text is outdated...

Whisperfish should be considered alpha software and used at your own risk. The
client is not an official Signal client and is not affiliated with Open Whisper
Systems. The code has not been audited by an expert in computer security or
cryptography. 

Whisperfish uses the `Signal client library for Go
<https://github.com/aebruno/textsecure>`_ and `Qt binding for Go
<https://github.com/therecipe/qt>`_.  The user interface is heavily based on
the jolla-messages application written by Jolla Ltd.
  
-------------------------------------------------------------------------------
Features
-------------------------------------------------------------------------------

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

-------------------------------------------------------------------------------
Nightly builds
-------------------------------------------------------------------------------

The most recent builds can be found here:

- armv7hl: https://gitlab.com/rubdos/whisperfish/-/jobs/artifacts/master/browse?job=build-arm
- i486: https://gitlab.com/rubdos/whisperfish/-/jobs/artifacts/master/browse?job=build-x86


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


1a. This application uses `libsignal-c-protocol
    <https://github.com/signalapp/libsignal-protocol-c>`_
    as a git submodule.::

    $ git clone --recurse-submodules https://github.com/rubdos/whisperfish/

1b. If you already had cloned the repository, you can use::

    $ git submodule update --init --recursive

2. Since that library is built using `cmake <https://cmake.org/>`_,
   we need cmake *in the build environment*.
   You can install it from within the SDK.
   While you're at it, install git too. `qmake` will embed the current git ref in the build name.
   If you prefer to install it over the command line, `ssh` into your build system and use `zypper`::

    $ ssh -p 2222 -i ~/SailfishOS/vmshare/ssh/private_keys/engine/mersdk mersdk@localhost
    $ sudo zypper -n install cmake make git

3. From here on, you can just use the SailfishOS SDK as per usual

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

Whisperfish is free software: you can redistribute it and/or modify it under the
terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
this program. If not, see <http://www.gnu.org/licenses/>.
