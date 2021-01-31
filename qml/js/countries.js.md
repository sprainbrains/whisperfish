# Country data

## Sources

- [CC](https://en.wikipedia.org/wiki/List_of_country_calling_codes#Alphabetical_listing_by_country_or_region) (2021-01-30)
- [ISO](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2#Officially_assigned_code_elements) (2021-01-31)

## Conversion

	--- cc, iso: clean up entries, remove line breaks etc.
	sed -Ee 's/(.*?)[ ]*\t(.*?)[ ]*\t.*$/  {n:"\1", p:"\2"},/g' cc_src > cc_raw
	--- cc: remove some numbers that are unlikely to be used for online chats (can be re-added)
	--- cc: split countries with multiple numbers into multiple entries
	grep $'\t' iso_src | cut -f1,2 | sed -Ee 's/(.*?)\t(.*?)/\2\t\1/g;s/ \t/\t/g;s/ $//g' | sort -t $'\t' -k1b,1 > iso
	grep -ve '^//' cc_raw | cut -d: -f2,3 | tail -n +2 | head -n -1 | sed -Ee 's/"(.*?)", p:"(.*?)"},?/\1\t\2/g;' | sort -t $'\t' -k1b,1 > cc
	join -t $'\t' -i -j 1 cc iso -o 1.1,2.2,1.2 > joined
	join -t $'\t' -i -j 1 cc iso -o 1.1,1.2 -v 1 > fail_cc
	join -t $'\t' -i -j 1 cc iso -o 2.1,2.2 -v 2 > fail_iso
	--- manually fix failed lines, think of https://xkcd.com/927/ and politics, hopefully no offence
	perl -pe 's/^(.*?)\t(.*?)\t(.*?)(\t.*?)?$/{n:"\1",i:"\2",p:"\3"},\4/g; s/,\t/, /g;' joined_fixed |\
		sort | sed '$a ]' | sed '1i var c=[//n=name,i=ISO,p=prefix -- see countries.js.md for source' > countries.js

**Important:** ISO may be empty, calling code must be available.

## Not included

(Probably) no calling codes:

	Bouvet Island	BV		UNINHABITED
	French Southern Territories	TF		ANTARCTICA
	Heard Island and McDonald Islands	HM		UNLCEAR
	Western Sahara	EH		UNCLEAR
	United States Minor Outlying Islands	UM		UNCLEAR

Non-geographic entities:

	Ellipso (Mobile Satellite service) 	+881 2, +881 3
	EMSAT (Mobile Satellite service) 	+882 13
	Global Mobile Satellite System (GMSS) 	+881
	Globalstar (Mobile Satellite Service) 	+881 8, +881 9
	ICO Global (Mobile Satellite Service) 	+881 0, +881 1
	Inmarsat SNAC 	+870
	International Freephone Service (UIFN) 	+800
	International Networks 	+882, +883
	International Premium Rate Service 	+979
	International Shared Cost Service (ISCS) 	+808
	Iridium (Mobile Satellite service) 	+881 6, +881 7
	Telecommunications for Disaster Relief by OCHA 	+888
	Thuraya (Mobile Satellite service) 	+882 16
	Universal Personal Telecommunications (UPT) 	+878
