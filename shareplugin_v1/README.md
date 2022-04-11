
# Whisperfish Transfer Plugin #

Adds "share via whisperfish" option to sailfish. See
https://git.sailfishos.org/mer-core/transfer-engine/tree/master for details.

## MOC ##

Since this is a qt plugin, we need to run moc before compiling. In our
cross-compiling setup, the host qt version might not be equal to the one used
by sailfish, so we can't use moc from the host. Therefore, we include the
generated sources for sailfish in the repo.

Please update the moc sources after changing one of the header files by running
the followin commands inside the sdk:

```
sb2 moc -I /usr/include -o sfmoc/WhisperfishTransfer.cpp WhisperfishTransfer.h
sb2 moc -I /usr/include -o sfmoc/WhisperfishTransferPlugin.cpp WhisperfishTransferPlugin.h
```

## Sailjail / Sailfish 4 ##

Applications in Sailjail, like jolla-gallery on sailfish 4, need additional
permissions to call Whisperfish. Add the following two lines to
`/etc/sailjail/permissions/Sharing.permission` allow them to share via
Whisperfish:

```
dbus-user.call be.rubdos.whisperfish=be.rubdos.whisperfish.app.handleShare@/be/rubdos/whisperfish/app
dbus-user.own be.rubdos.whisperfish.shareClient.*
```
