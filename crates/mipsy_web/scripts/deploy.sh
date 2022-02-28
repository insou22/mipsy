#!/bin/bash
# Note, this presumes you have an ssh alias `cse` 


if ! command -v wasm-pack 2>&1 >/dev/null;
then
	echo 'error: you must install wasm-pack (try `cargo install wasm-pack`)'
	exit
fi

if ! command -v tailwindcss 2>&1 >/dev/null;
then
	echo 'error: you must install tailwindcss (try `npm i -g tailwindcss`)'
	exit
fi

wasm-pack build --target no-modules --out-name wasm --out-dir ./dist --no-typescript

cd dist
ln -sf ../_static/index.html
cd ..
cp -r _static/package/ dist

NODE_ENV=production tailwindcss -c ./tailwind.config.js -o dist/tailwind.css --minify

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
