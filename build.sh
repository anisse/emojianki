#!/bin/bash

mkdir -p web/cldr/{annotations,annotationsDerived,main}
for file in cldr/common/annotationsDerived/*
do
	lang="$(basename -s .xml "$file")"
	for dest in annotations annotationsDerived main
	do
		cp "cldr/common/$dest/$lang.xml" web/cldr/$dest/
	done
done
