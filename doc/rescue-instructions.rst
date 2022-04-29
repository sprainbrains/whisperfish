How to rescue attachments (in vim)
==================================

Logging is not enabled by default, but if you
suspect you may be losing attachments or are
behind a bad connection that causes problems,
you should run Whispefish with the ``--verbose``
option.

Until this is a first-class feature, you can make
a script like

.. code:: bash

        #!/usr/bin/env bash

        RUST_BACKTRACE=trace invoker --type=qtquick2 /usr/bin/harbour-whisperfish --verbose

to launch with.

The file will end up in ``/home/nemo/.local/share/harbour-whisperfish/``.

Required function
-----------------

::

        "" https://vim.fandom.com/wiki/Convert_between_hex_and_decimal
        " Adapted hex format
        function! Dec2hex(arg)
          return printf('%02x', a:arg + 0)
        endfunction

Fetcher
-------

Send it off to your phone, it will naturally be required later on.
``scp -p ./target/armv7-unknown-linux-gnueabihf/release/fetch-signal-attachment nemo@${SSH_TARGET}:``

::

        scp -p target/armv7-unknown-linux-gnueabihf/release/fetch-signal-attachment nemo@${SSH_TARGET}:

Steps
-----

Copy the attachment log file you want to manipulate onto your computer.

The file is like ``.local/share/harbour-whisperfish/attachments-2021-04-15_15-47.log``.

Protip: steps 1-7 can be a macro recorded with ``qa``, then the line-specific ones ``qb`` and ``qc``,
with the rest ``qd``!

Write and open a copy like ``attachments-2021-01-24_10-36.log.rescue``

  1. Remove all irrelevant lines
     ``:%v/Error fetching attachment/d``
  2. Replace what's leading up to and around the message ID
     ``:%s#.*for message with ID `\(\d\+\)` AttachmentPointer {#$HOME/fetch-signal-attachment --message-id \1#g``
  3. ``jpeg`` becomes ``jpg`` just in case:
     ``:%s/jpeg/jpg/g``
  4. Also plaintext:
     ``:%s/Some("text\/x-signal-plain")/Some("text\/plain")/g``
  5. Format the extension part
     ``:%s/content_type: [^"]*"\([^/]\+\)\/\(\w\+\)"),/--ext \2 --mime-type \1\/\2/g``
  6. Also plaintext must be fixed if required
     ``:%s/--ext plain/--ext txt/g``
  7. Format the key part, still in decimal, by grabbing that relevant part
     ``:%s/key:[^(]*(\([^)]*\)), /--key \1 /g``
  8. ``CdnId`` and ``CdnKey`` used interchangeably in the logs, but the fetcher wants
      it like the key: ``:%s/CdnId(\(\d\+\))/CdnKey("\1")/g``
      (said Ruben, dunno lol, let's see, breaks Vim macros if it doesn't exist)

  9. Get tired of wranging vim and do the hex conversion manually; on every line run
      ``f[v%`` and ``:s/\%V\(\d\+\)/\=Dec2hex(submatch(0))/g`` and make it a macro if you want
  10. Still tired of wranging vim and do space/comma removal manually; on every line run
      ``f[v%`` and ``:s/\%V[, ]//g`` and make it a macro if you want
  11. Get rid of brackets ``:%s/\(^[^[]*\)\[\([^]]*\)\]/\1\2``
  12. ``:%s/size:.*cdn_number[^\d]\+\(\d\+\)),/--cdn-number \1/g`` to connect to the right place
  13. ``:%s/size:.*cdn_number: None,/--cdn-number 0/g`` to connect to the right place
  14. ``:%s/attachment_identifier:.*CdnKey("\(.*\)").*/--cdn-key \1/g`` to rule them all

Now time for the great scriptification!

Also set the password ``:%s/$/ --password your_password_here/g`` with the warning
that it might end up in your history/undo files, so you can also set the password
using ssh+vim on your phone when all this is done.

Sort by message id ``:w | !sort -k3 % | tee > %.foo && mv %.foo %``.

Make it executable; ``:!chmod +x %``.

If you have any ``None``, before the first, add ``exit 0``, because everything
will fail catastrophically after that point.

Write this at the top

::

        #!/usr/bin/env bash

        set -eux

        export RUST_LOG=info

        cd ~/Pictures/Whisperfish/attachments/

Copy it over to your phone; the home directory's a fine location for it.

If you didn't set your password into the file, edit it on the phone and add it.

On the phone
------------

Just run this

::

        ./attachments-2021-01-24_10-36.log.rescue

