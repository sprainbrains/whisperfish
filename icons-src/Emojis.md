# Emojis

Whisperfish supports different emoji styles. They have to be installed in:

    <app data>/icons/emojis/<style>/<version>/

Loading emojis from `~/.local/share/...` is not yet supported.

Currently, all styles have to be registered in `qml/js/emoji.js`. This can be
simplified in the future.

Emoji sets are folders of emoji icons in either a vector or raster format (`svg`
is preferred). Each emoji icon must be named `<codepoint>.<ext>` (lower case),
combined codepoints are separated by `-`. Some emojis may include the "emoji
style" selector (`uFE0F`); it is recommended to create symbolic links for the
version with and without the selector.


## OpenMoji

- source: https://github.com/hfg-gmuend/openmoji/releases/tag/13.0.0
          https://github.com/hfg-gmuend/openmoji/releases/download/13.0.0/openmoji-svg-color.zip
- license: CC-BY-SA 4.0
- format: svg
- style: color (lively) and/or black

Note: black is not recommended because emojis are not highlighted according to
Sailfish theme colors. Black emojis may be invisible in dark ambiences.

**Installation:** extract the release ball to `openmoji/13.0.0`, then convert
all file names to lower case (requires `bash`):

    for i in *.svg; do mv "$i" "${i,,}"; done


## Twitter

- source: https://github.com/twitter/twemoji/releases/tag/v13.0.1
          https://github.com/twitter/twemoji/archive/v13.0.1.tar.gz
- license: CC-BY-SA 4.0
- format: svg
- style: color (flat)

**Installation:** extract `assets/svg/` from the release ball to `twemoji/13.0.1`


## Emojipedia

Emojipedia provides all emojis in many different proprietary styles. The
example below uses "Whatsapp" but should work for all styles.

- source: https://emojipedia.org/whatsapp/
          https://emojipedia.org/whatsapp/2.20.206.24/
- license: proprietary
- format: png
- style: color (glossy, different)

**Installation:**

1. Fetch emojis

    # fetch list of emojis
    curl 'https://emojipedia.org/whatsapp/2.20.206.24/' -H 'User-Agent: Mozilla/5.0 (Windows NT 10.0; rv:78.0) Gecko/20100101 Firefox/78.0' \
        -H 'Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8' \
        -H 'Accept-Language: en-US;q=0.7,en;q=0.3' --compressed -H 'DNT: 1' -H 'Connection: keep-alive' \
        -H 'Upgrade-Insecure-Requests: 1' -H 'Sec-GPC: 1' -H 'Pragma: no-cache' -H 'Cache-Control: no-cache' > wa.html

    # extract links
    # available sizes: 60, 72, 120, 144, 160
    # default size appears to be 72px
    grep -Pe 'src="https://emojipedia-us\..*?\.amazonaws\.com/thumbs/72/whatsapp/.*?/.*?.png"' wa.html -o | sed 's/^src="//g;s/"$//g' > links

    # fetch files
    # use $(cat links) in curl or use xargs
    xargs -a links -I{} curl -H 'User-Agent: Mozilla/5.0 (Windows NT 10.0; rv:78.0) Gecko/20100101 Firefox/78.0' \
        -H 'Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8' \
        -H 'Accept-Language: en-US;q=0.7,en;q=0.3' --compressed -H 'DNT: 1' -H 'Connection: keep-alive' \
        -H 'Upgrade-Insecure-Requests: 1' -H 'Sec-GPC: 1' -H 'Pragma: no-cache' -H 'Cache-Control: no-cache' \
        --remote-name-all {}

2. Fix file names and alternatives

    for i in *.png; do mv "$i" "${i#*_}"; done
    for i in *-skin-tone_*.png; do mv "$i" "${i#*-skin-tone_}"; done
    for i in *_1f3f*.png; do mv "$i" "${i%_1f3f*.png}.png"; done

    # handle optional "emoji style" selectors
    for i in *-fe0f.png; do ln -b -s "$i" "${i%-fe0f.png}.png"; done

    # *OR* in one line
    for i in *.png; do mv "$i" "${i#*_}"; done && \
        for i in *-skin-tone_*.png; do mv "$i" "${i#*-skin-tone_}"; done && \
        for i in *_1f3f*.png; do mv "$i" "${i%_1f3f*.png}.png"; done && \
        for i in *-fe0f.png; do ln -b -s "$i" "${i%-fe0f.png}.png"; done

3. Move files to `whatsapp/2.20.206.24`
