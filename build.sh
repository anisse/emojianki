#!/bin/bash

count=$(echo cldr/common/annotationsDerived/* |wc -w)
echo "pub(crate) static LANGUAGES: [&str; $count] = [" > src/available.rs
mkdir -p web/cldr/{annotations,annotationsDerived,main}
for file in cldr/common/annotationsDerived/*
do
	lang="$(basename -s .xml "$file")"
	echo "    \"$lang\"," >> src/available.rs
	for dest in annotations annotationsDerived main
	do
		cp "cldr/common/$dest/$lang.xml" web/cldr/$dest/
	done
done
echo "];" >> src/available.rs
