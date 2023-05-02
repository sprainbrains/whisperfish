#!/bin/bash

declare -A languages=( [ca]="Catalan" [cs]="Czech" [de]="German" [el]="Greek" [es]="Spanish" [eu]="Basque" [fi]="Finnish" [fr]="French" [hu]="Hungarian" [it]="Italian" [lt]="Lithuanian" [nb_NO]="Norwegian Bokmål" [nl_BE]="Dutch (Belgium)" [nl]="Dutch" [pl]="Polish" [pt_BR]="Portuguese (Brazil)" [pt_PT]="Portuguese (Portugal)" [ro]="Romanian" [ru]="Russian" [sl]="Slovenian" [sv]="Swedish" [tr]="Turkish" [zh_CN]="Chinese (Simplified)")


# These people regularly touch .ts files and will show up literally in every translation.
IGNORE=$'Gabriel Margiani\nRuben De Smet\nMatti Viljanen\nMirian Margiani\nMarkus Törnqvist\nGitlab CI translation file sync'

ABOUT=qml/pages/About.qml

PREFIX="                    "

for key in "${!languages[@]}"; do
    lang=${languages[$key]}

    echo "$PREFIX""//: Name of the $lang ($key) language, about page translation section";
    echo "$PREFIX""//% \"$lang\"";
    echo "$PREFIX""qsTrId(\"whisperfish-lang-$key\") + \": \" +";

    while IFS= read -r contributor; do
        contributor=$(echo -n $contributor | awk '{sub($1 OFS, "")}1');

        while IFS= read -r non_contributor; do
            if [ "$contributor" = "$non_contributor" ] ; then
                continue 2;
            fi
        done <<< "$IGNORE"

        echo "$PREFIX\"$contributor,\" +"

    done <<< "`git shortlog -s -n translations/harbour-whisperfish-$key.ts`"
    echo "$PREFIX\"\\n\" +"
done > TRANSLATORS.tmp
echo "$PREFIX\"\"" >> TRANSLATORS.tmp

text=$(cat TRANSLATORS.tmp)
rm TRANSLATORS.tmp

start=`sed -e '/BEGIN TRANSLATORS/q' $ABOUT`

end=`sed -ne '/END TRANSLATORS/,$ p' $ABOUT`

echo "$start" > $ABOUT
echo "                ""text: {" >> $ABOUT
echo "$text" >> $ABOUT
echo "                ""}" >> $ABOUT
echo "$end" >> $ABOUT
