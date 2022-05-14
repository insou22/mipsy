#!/bin/bash
# Note, this presumes you have an ssh alias `cse` 


if ! command -v trunk 2>&1 >/dev/null;
then
	echo 'error: you must install trunk (try the instructions at https://trunkrs.dev/)'
	exit
fi

if ! command -v tailwindcss 2>&1 >/dev/null;
then
	echo 'error: you must install tailwindcss (try `npm i -g tailwindcss`)'
	exit
fi

trunk build --release

if [ "$1" = "--push=shreys" ]; then

    scp -r dist/* cse:~/web/mipsy/

fi

if [ "$1" = "--push=cs1521" ]; then

    scp -r dist/* cse:~cs1521/web/mipsy/

fi

if [ "$1" = "--push=both" ]; then
    scp -r dist/* cse:~cs1521/web/mipsy
    scp -r dist/* cse:~/web/mipsy
fi
