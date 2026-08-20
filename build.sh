#!/bin/bash

mkdir -p web/cldr/{annotations,annotationsDerived}
for file in cldr/common/annotationsDerived/*
do
	lang="$(basename -s .xml "$file")"
	for dest in annotations annotationsDerived
	do
		cp "cldr/common/$dest/$lang.xml" web/cldr/$dest/
	done
done
